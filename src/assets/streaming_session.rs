//It can be done better. The current variant is temp


use std::collections::{HashMap, HashSet};
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::Arc;

use crate::assets::mesh_data_manager::{DataManager};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::oneshot::Receiver;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;

pub struct UserRequest {
    pub id: u64,
}

struct InternalRequest {
    req: UserRequest,
    oneshot: oneshot::Sender<Response>,
    is_shutdown: bool,
}

pub struct Response {
    pub data: Option<Arc<Vec<u8>>>,
}

pub struct Cache {
    data: HashMap<u64, Arc<Vec<u8>>>,
    cleaning_task: Option<JoinHandle<()>>,
    // cache_max_size: usize,
    // keys: VecDeque<u64>,

    //  current_size: usize,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            cleaning_task: None,
            //    cache_max_size,
            // keys: Default::default(),
            //current_size: 0,
        }
    }

    pub fn try_get_resource(&self, resource: u64) -> Option<Arc<Vec<u8>>> {
        let res = if let Some(data) = self.data.get(&resource) {
            Some(data.clone())
        } else {
            None
        };

        res
    }

    pub async fn insert_resource(&mut self, id: u64, data: Arc<Vec<u8>>) {
        if !self.data.contains_key(&id) {
            //  self.current_size += data.len();
            self.data.insert(id, data);
            //self.keys.push_back(id);
            // self.clean_by_size().await;
        }
    }

    pub async fn try_cleanup_cache(&mut self, self_ref: Arc<Mutex<Self>>) {
        if let Some(cleaning_task) = self.cleaning_task.as_mut() {
            if cleaning_task.is_finished() {
                self.cleaning_task = None;
            } else {
                return;
            }
        }

        self.cleaning_task = Some(tokio::task::spawn(async move {
            self_ref.lock().await.cleanup_cache().await;
        }))
    }

    /** Borked

       async fn clean_by_size(&mut self) {
           if self.cache_max_size != 0 {
               while self.current_size > self.cache_max_size {
                   if let Some(key) = self.keys.pop_front() {
                       if let Some(data) = self.data.remove(&key){
                           self.current_size -= data.len();
                       }
                   } else {
                       break;
                   }
               }
           }
       }


     */
    async fn cleanup_cache(&mut self) {
        let mut to_remove = HashSet::new();
        for (key, data) in self.data.iter() {
            if Arc::strong_count(data) == 1 {
                to_remove.insert(key.clone());
            }
        }
        //   self.remove_key_entries(&to_remove).await;
        to_remove.iter().for_each(|key| {
            if let Some(data) = self.data.remove(key) {
                //   self.current_size -= data.len();
            }
        });
        //  self.clean_by_size().await;
    }
    /*
    async fn remove_key_entries(&mut self, keys: &HashSet<u64>) {
        self.keys.retain(|k| !keys.contains(k));
    }

     */
}

pub struct StreamingSession {
    data_manager: Option<DataManager>,
    send_channel: mpsc::Sender<InternalRequest>,
    recv_channel: Option<mpsc::Receiver<InternalRequest>>,
    main_task: Option<JoinHandle<()>>,
}

impl StreamingSession {
    pub fn new(
        data_manager: DataManager,
        max_requests_at_time: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<InternalRequest>(max_requests_at_time);
        Self {
            data_manager: Some(data_manager),
            send_channel: tx,
            recv_channel: Some(rx),
            main_task: None,
        }
    }

    pub async fn run(&mut self) -> Option<()> {
        if self.main_task.is_some() {
            return None;
        }

        let recv = self.recv_channel.take()?;
        let data_manager = self.data_manager.take()?;
        let _ = self.main_task.replace(tokio::spawn(async move {
            Self::loader_main(data_manager, recv).await;
        }));
        Some(())
    }

    pub async fn dispatch_task(&self, request: UserRequest) -> Option<Receiver<Response>> {
        let (tx, rx) = oneshot::channel();
        let int_request = InternalRequest {
            req: request,
            oneshot: tx,
            is_shutdown: false,
        };
        self.send_channel.send(int_request).await.ok()?;
        Some(rx)
    }

    pub async fn stop(&self) {
        let (tx, rx) = oneshot::channel();
        let int_request = InternalRequest {
            req: UserRequest { id: 0 },
            oneshot: tx,
            is_shutdown: true,
        };
        self.send_channel.send(int_request).await.ok().unwrap();
    }

    async fn loader_main(
        data_manager: DataManager,
        mut recv: mpsc::Receiver<InternalRequest>,
    ) {
        let data_manager = Arc::new(data_manager);
        let cache: Arc<Mutex<Cache>> = Arc::new(Mutex::new(Cache::new()));
        while let Some(req) = recv.recv().await {
            if req.is_shutdown {
                break;
            }
            let data_manager = data_manager.clone();
            let cache = cache.clone();
            let _ = tokio::spawn(async move {
                Self::process_request(req, data_manager, cache).await;
            });
        }
    }

    async fn process_request(
        request: InternalRequest,
        data_manager: Arc<DataManager>,
        cache: Arc<Mutex<Cache>>,
    ) {
        if let Some(data) = cache.lock().await.try_get_resource(request.req.id) {
            let _ = request.oneshot.send(Response { data: Some(data) });
            return;
        }

        let meta = data_manager.get_binary_meta(request.req.id);
        if let Some(meta) = meta {
            let storage_meta = data_manager.get_data_location(request.req.id);
            if let Some(storage) = storage_meta {
                if let Some(data) =
                    Self::load_binary(&storage.0, request.req.id, meta.offset, meta.size, cache)
                        .await
                {
                    let _ = request.oneshot.send(Response { data: Some(data) });
                    return;
                }
            }
        }
        let _ = request.oneshot.send(Response { data: None });
    }

    async fn load_binary(
        path: &PathBuf,
        id: u64,
        offset: usize,
        size: usize,
        cache: Arc<Mutex<Cache>>,
    ) -> Option<Arc<Vec<u8>>> {
        let cache_clone = cache.clone();
        cache.lock().await.try_cleanup_cache(cache_clone).await;
        let mut res = Vec::with_capacity(size);
        unsafe {
            res.set_len(size);
        }
        let mut file = File::open(path).await.ok()?;
        file.seek(SeekFrom::Start(offset as u64)).await.ok()?;
        file.read_exact(res.as_mut_slice()).await.ok()?;

        let data = Arc::new(res);

        cache.lock().await.insert_resource(id, data.clone()).await;

        Some(data)
    }
}