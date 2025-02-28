use actix::Actor;
use serde_json::json;
use shiny_rs::shiny_rs_derive::ShinyHandler;
use shiny_rs::changed;
use shiny_rs::session::*;
use shiny_rs::session::input_pool::InputPool;
use shiny_rs::session::traits::*;
use shiny_rs::ui::*;
use std::time::Instant;
use comrak::{ markdown_to_html, ComrakOptions };

use super::plot::{ get_plot, get_dist };

fn sample_dist(n: u64, mean: f64, sd: f64) -> Vec<f64> {
    get_dist(n as usize, mean, sd).unwrap_or_default()
}

fn build_plot(session: &mut CustomSession, dist1: &[f64], dist2: &[f64]) {
    let my_plot = get_plot(dist1, dist2);
    render_ui(session, "plot1", &my_plot);
}

fn validate_range(session: &mut CustomSession, n: u64) -> bool {
    if (1..=10000).contains(&n) {
        true
    } else {
        show_notification(
            session,
            json!({
                "html": "Number out of range",
                "action": "",
                "deps": [],
                "closeButton": true,
                "id": generate_id(),
                "type": "error"
            })
        );
        false
    }
}

#[derive(ShinyHandler)]
pub struct CustomServer {
    hb: Instant,
    pub input: InputPool,
    pub event: String,
    initialize: fn(&mut Self, session: &mut <Self as Actor>::Context),
    update: fn(&mut Self, session: &mut <Self as Actor>::Context),
    tick: fn(&mut Self, session: &mut <Self as Actor>::Context),
    dist1: Vec<f64>,
    dist2: Vec<f64>,
    hb_interval: std::time::Duration,
    client_timeout: std::time::Duration
}

impl CustomServer {
    pub fn new(
        initialize: fn(&mut Self, session: &mut <Self as Actor>::Context),
        update: fn(&mut Self, session: &mut <Self as Actor>::Context),
        tick: fn(&mut Self, session: &mut <Self as Actor>::Context),
    ) -> Self {
        CustomServer {
            hb: Instant::now(),
            input: InputPool::new(),
            event: String::from("Init"),
            dist1: vec!(),
            dist2: vec!(),
            initialize,
            update,
            tick,
            hb_interval: std::time::Duration::from_secs(5),
            client_timeout: std::time::Duration::from_secs(10),
        }
    }
}

impl Actor for CustomServer {
    type Context = ShinyContext<Self>;
    fn started(&mut self, session: &mut Self::Context) {
        self.hb(session);
    }
}

type CustomSession = ShinyContext<CustomServer>;

pub fn initialize(shiny: &mut CustomServer, session: &mut CustomSession) {
    shiny.dist1 = sample_dist(
        shiny.input.get_u64("n-1:shiny.number").unwrap_or(0),
        shiny.input.get_f64("mean-1:shiny.number").unwrap_or(0.0),
        shiny.input.get_f64("sd-1:shiny.number").unwrap_or(0.1)
    );
    shiny.dist2 = sample_dist(
        shiny.input.get_u64("n-2:shiny.number").unwrap_or(0),
        shiny.input.get_f64("mean-2:shiny.number").unwrap_or(0.0),
        shiny.input.get_f64("sd-2:shiny.number").unwrap_or(0.1)
    );
    build_plot(session, &shiny.dist1, &shiny.dist2);
}

pub fn update(shiny: &mut CustomServer, session: &mut CustomSession) {
    if changed!(shiny, ("markdown")) {
        let md_string = shiny.input.get_string("markdown").unwrap_or_default();
        if md_string.len() > 5000 {
            show_notification(session, args!({
                "html": "Exceeded 5,000 characters!",
                "id": "markdown_warning",
                "type": "error",
                "closeButton": true
            }));
        }
        let render = markdown_to_html(&md_string, &ComrakOptions::default());
        render_ui(session, "rendered_md", &render);
    }
    if changed!(shiny, ("insert_ui:shiny.action")) {
        let dist1 = sample_dist(50, -1.0, 0.5);
        let dist2 = sample_dist(50, -1.0, 0.5);
        insert_ui(
            session,
            "#insert_section",
            "afterBegin",
            &get_plot(&dist1, &dist2)
        )
    }
    if changed!(shiny, ("remove_ui:shiny.action")) {
        remove_ui(session, "#insert_section div")
    }
    if changed!(shiny, ("n-1:shiny.number", "mean-1:shiny.number", "sd-1:shiny.number")) {
        let n = shiny.input.get_u64("n-1:shiny.number").unwrap_or(0);
        if validate_range(session, n) {
            shiny.dist1 = sample_dist(
                n,
                shiny.input.get_f64("mean-1:shiny.number").unwrap_or(0.0),
                shiny.input.get_f64("sd-1:shiny.number").unwrap_or(0.1)
            )
        }
        build_plot(session, &shiny.dist1, &shiny.dist2);
    }
    if changed!(shiny, ("n-2:shiny.number", "mean-2:shiny.number", "sd-2:shiny.number")) {
        let n = shiny.input.get_u64("n-2:shiny.number").unwrap_or(0);
        if validate_range(session, n) {
            shiny.dist2 = sample_dist(
                n,
                shiny.input.get_f64("mean-2:shiny.number").unwrap_or(0.0),
                shiny.input.get_f64("sd-2:shiny.number").unwrap_or(0.1)
            )
        }
        build_plot(session, &shiny.dist1, &shiny.dist2);
    }
    if changed!(shiny, ("text1")) {
        let val = shiny.input.get_string("text1").unwrap_or_default();
        update_text_input(
            session,
            "text2",
            json!({
                "label": val
            })
        )
    }
    if changed!(shiny, ("text2")) {
        let val = shiny.input.get_string("text2").unwrap_or_default();
        update_text_input(
            session,
            "text1",
            json!({
                "label": val
            })
        )
    }
}

pub fn tick(_shiny: &mut CustomServer, _session: &mut CustomSession) {
}

pub fn create_server() -> CustomServer {
    CustomServer::new(initialize, update, tick)
}
