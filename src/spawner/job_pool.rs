use threadpool::ThreadPool;
use std::{marker::PhantomData, path::{Path, PathBuf}, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}};

pub struct NoneJob;
pub struct DefinedJob;


pub type JobFn<T> = Box<dyn Fn(T) -> Result<(), String> + Send + Sync + 'static>;
type WorkerFn = Box<dyn FnOnce() + Send + Sync + 'static>;


pub struct SIJPool<T>{
    _type_phantom: PhantomData<T>
}

impl<T> SIJPool<T>{
    pub fn new(worker_no: usize) -> SIJPoolBuilder<T, NoneJob>{
        SIJPoolBuilder::<T, NoneJob> { 
            job: None, 
            worker_no, 
            path_report: None,
            _job_define: PhantomData
        }
    }
}



/// Single Instruction Job Pool Builder
/// Creates a thread pool that have the same job
/// 
pub struct SIJPoolBuilder<T, J>{
    job: Option<JobFn<T>>,
    worker_no: usize,
    path_report: Option<PathBuf>, 
    _job_define: PhantomData<J>
}

pub struct SIJChannelHandler<T>{
    sender: Sender<T>,
    thread_pool: ThreadPool
}

impl<T, J> SIJPoolBuilder<T, J>
where T: Send + 'static
{

    pub fn report_path(self, path: PathBuf ) -> SIJPoolBuilder<T, J> {
        SIJPoolBuilder::<T, J> { 
            path_report: Some(path),
            ..self
        }
    }

    fn build_job_instruction(&self, arc_closure: Arc<JobFn<T>>, arc_rx: Arc<Mutex<Receiver<T>>>) -> WorkerFn {
        if self.path_report.is_some() {
            
            Box::new( move || loop {
                let Ok(mutex) = arc_rx.lock() else {
                    return;
                };

                let Ok(t_data) = mutex.recv() else {
                    return;
                };

                let _ = arc_closure(t_data);

                // do some job error output in 
                // given report path



            })
        } else {
            Box::new( move || loop {
                let Ok(mutex) = arc_rx.lock() else {
                    return;
                };

                let Ok(t_data) = mutex.recv() else {
                    return;
                };

                let _ = arc_closure(t_data);


            })
        }
    }

}

impl<T> SIJPoolBuilder<T, NoneJob>
where T: Send + 'static
{
    
    pub fn set_job(self, op: JobFn<T>) -> SIJPoolBuilder::<T, DefinedJob>{
        SIJPoolBuilder::<T, DefinedJob>{
            job: Some(op),
            worker_no: self.worker_no,
            path_report: self.path_report,
            _job_define: PhantomData
        }
    }

}

impl<T> SIJPoolBuilder<T, DefinedJob>
where T: Send + 'static
{

    pub fn build(self) -> SIJChannelHandler<T>{
        todo!()        
    }

    

}



