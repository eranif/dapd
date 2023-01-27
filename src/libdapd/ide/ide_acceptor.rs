use crate::libdapd::DriverGDB;
use crate::requests::LaunchRequestArguments;
use async_trait::async_trait;
use dap::prelude::*;
use thiserror::Error;

pub struct IdeAcceptor {
    pub launch_arguments: LaunchRequestArguments,
    pub gdb: DriverGDB,
}

impl Default for IdeAcceptor {
    fn default() -> Self {
        Self {
            launch_arguments: LaunchRequestArguments {
                no_debug: None,
                restart_data: None,
                program: None,
                args: None,
                cwd: None,
                env: None,
            },
            gdb: DriverGDB::new(),
        }
    }
}

impl IdeAcceptor {
    /// `Clone` the launch arguments (no Clone()) is available for
    /// the struct `LaunchRequestArguments`
    fn set_launch_arguments(&mut self, args: &LaunchRequestArguments) {
        self.launch_arguments = args.clone();
    }
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum IdeAcceptorError {
    #[error("Unhandled command")]
    UnhandledCommandError,
}

#[async_trait]
impl Adapter for IdeAcceptor {
    type Error = IdeAcceptorError;

    /// the main event loop
    /// the server will call this per command received from the client
    async fn handle_request(
        &mut self,
        request: Request,
        _client: &mut StdoutWriter,
    ) -> Result<Response, Self::Error> {
        eprintln!("Accept {:?}\n", request.command);

        match &request.command {
            Command::Initialize(args) => {
                if let Some(client_name) = args.client_name.as_ref() {
                    eprintln!("> Client '{client_name}' requested initialization.");
                    Ok(Response::make_success(
                        &request,
                        // for now, these are the capabilities we support
                        ResponseBody::Initialize(Some(types::Capabilities {
                            supports_configuration_done_request: Some(true),
                            supports_evaluate_for_hovers: Some(true),
                            supports_function_breakpoints: Some(true),
                            ..Default::default()
                        })),
                    ))
                } else {
                    Ok(Response::make_error(&request, "Missing client name"))
                }
            }
            Command::Launch(args) => {
                eprintln!("> Launch called");
                // keep the launch arguments
                self.set_launch_arguments(&args);
                if let Err(e) = self.gdb.launch(&args).await {
                    Ok(Response::make_error(
                        &request,
                        &format!("Failed to launch gdb. {:?}", e),
                    ))
                } else {
                    Ok(Response::make_success(&request, ResponseBody::Launch))
                }
            }
            Command::SetBreakpoints(_args) => {
                eprintln!("> SetBreakpoints called");
                Ok(Response::make_error(
                    &request,
                    "command SetBreakpoints unsupported",
                ))
            }
            Command::SetFunctionBreakpoints(_args) => {
                eprintln!("> SetFunctionBreakpoints called");
                Ok(Response::make_error(
                    &request,
                    "command SetFunctionBreakpoints unsupported",
                ))
            }
            Command::Next(_) => Ok(Response::make_ack(&request).unwrap()),
            _ => {
                eprintln!("> received unsupported command");
                Ok(Response::make_error(&request, "command unsupported"))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::libdapd::run_async;

    /// construct DP request from the payload
    fn build_dap_request(payload: &str) -> Request {
        let request = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(e) => {
                panic!("{:?}", e);
            }
        };
        Request::from(request)
    }

    fn to_json(response: &Response) -> String {
        let r = match serde_json::to_string(response) {
            Ok(s) => s,
            Err(e) => {
                panic!("{:?}", e);
            }
        };
        r
    }

    #[test]
    fn test_initialize_request() -> Result<(), Box<dyn std::error::Error>> {
        run_async(async move {
            let init_request = r#"{
  "arguments": {
    "adapterID": "my-id",
    "clientID": "wxdap",
    "clientName": "wxdap",
    "columnsStartAt1": false,
    "linesStartAt1": false,
    "locale": "en-US",
    "pathFormat": "path",
    "supportsInvalidatedEvent": false
  },
  "command": "initialize",
  "seq": 1,
  "type": "request"
}"#;
            let request = build_dap_request(init_request);

            let mut acceptor = IdeAcceptor::default();
            let mut transport = StdoutWriter::new();
            let Ok(res) = acceptor.handle_request(request, &mut transport).await else {
            panic!("failed to process request");
        };
            assert!(res.success);
            let as_str = to_json(&res);
            assert!(as_str.contains("request_seq"));
            let _ = transport.send_response(res);
        })
    }

    #[test]
    fn test_launch_request() -> Result<(), Box<dyn std::error::Error>> {
        run_async(async move {
            let init_request = r#"{
  "arguments": {
    "args": [],
    "cwd": "/home/eran/wd",
    "env": [
      "SHELL=CMD.EXE",
      "CodeLiteDir=/home/eran/devl/codelite/build-release/install",
      "WXCFG=clang_x64_dll/mswu",
      "WXWIN=/home/eran/root"
    ],
    "noDebug": false,
    "program": "C:/Users/eran/Documents/TestWxCrafter/build-Debug/bin/TestWxCrafter.exe"
  },
  "command": "launch",
  "seq": 2,
  "type": "request"
}"#;
            let request = build_dap_request(init_request);

            let mut acceptor = IdeAcceptor::default();
            let mut transport = StdoutWriter::new();
            let Ok(res) = acceptor.handle_request(request, &mut transport).await else {
            panic!("failed to process request");
        };
            assert!(res.success);
            assert_eq!(acceptor.launch_arguments.no_debug, Some(false));
            assert_eq!(
                acceptor.launch_arguments.cwd,
                Some("/home/eran/wd".to_string())
            );

            let env = acceptor
                .launch_arguments
                .env
                .expect("expected non None environment!");

            assert_eq!(env.len(), 4);
            assert_eq!(env[0], "SHELL=CMD.EXE".to_string());
            assert_eq!(
                env[1],
                "CodeLiteDir=/home/eran/devl/codelite/build-release/install".to_string()
            );
            assert_eq!(env[2], "WXCFG=clang_x64_dll/mswu".to_string());
            assert_eq!(env[3], "WXWIN=/home/eran/root".to_string());
        })
    }
}
