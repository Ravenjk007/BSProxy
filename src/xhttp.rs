use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::sync::Arc;
use std::collections::HashMap;

pub struct XHTTPHandler {
    routes: Arc<HashMap<String, RouteHandler>>,
}

type RouteHandler = Box<dyn Fn(Request<Body>) -> Result<Response<Body>, hyper::Error> + Send + Sync>;

impl XHTTPHandler {
    pub fn new() -> Self {
        let mut routes = HashMap::new();
        
        // Rota padrão para teste
        routes.insert("/".to_string(), Box::new(|_req| {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("X-Status", "Multi-Protocol")
                .header("X-Supported", "SSL, SSH, WebSocket, XHTTP")
                .body(Body::from("BSProxy Multi-Protocol Server"))
                .unwrap())
        }));

        Self {
            routes: Arc::new(routes),
        }
    }

    pub async fn run(self, port: u16) -> Result<(), anyhow::Error> {
        let routes = self.routes.clone();
        
        let make_svc = make_service_fn(move |_conn| {
            let routes = routes.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let routes = routes.clone();
                    async move {
                        let path = req.uri().path().to_string();
                        
                        if let Some(handler) = routes.get(&path) {
                            return handler(req);
                        }
                        
                        // Rota não encontrada - 404
                        Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("Route not found"))
                            .unwrap())
                    }
                }))
            }
        });

        let addr = ([0, 0, 0, 0], port).into();
        let server = Server::bind(&addr).serve(make_svc);
        
        log::info!("XHTTP server running on port {}", port);
        server.await?;
        
        Ok(())
    }
}
