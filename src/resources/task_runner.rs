use actix::prelude::{Actor, Context, AsyncContext};
use actix::Arbiter;
use actix_web::web::Data;

use std::time::Duration;
use crate::types::AppState;

pub struct TaskRunner {
    arbiter: Arbiter,
    app_state: Data<AppState>
}

const RUN_INTERVAL_SEC: u64 = 15;

impl TaskRunner {
    pub fn new(app_state: Data<AppState>) -> TaskRunner {
        TaskRunner {
            arbiter: Arbiter::new(),
            app_state
        }
    }
}

impl Actor for TaskRunner {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("TaskRunner is alive");

        ctx.run_interval(Duration::from_secs(RUN_INTERVAL_SEC), move |this, _ctx| {
            let app_state = this.app_state.clone();
            this.arbiter.spawn(async move {
                app_state.service_registry.run().await.expect("service registry failed to execute!");
            });
        });
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("TaskRunner is stopped");
    }
}
