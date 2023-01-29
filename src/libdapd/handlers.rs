use crate::libdapd::DriverGDB;
use dap::client::Sendable;
use dap::prelude::*;
use gdb::Record;
use tokio::sync::mpsc::Receiver;

/// handle `InitializeRequest`
pub async fn handle_initialise(request: Request, stdout: &mut StdoutWriter) {
    let Command::Initialize(args) = &request.command else {
        panic!("expected Command::Initialize!");
    };
    let response = if let Some(client_name) = args.client_name.as_ref() {
        tracing::debug!("> Client '{client_name}' requested initialization.");
        Response::make_success(
            &request,
            // for now, these are the capabilities we support
            ResponseBody::Initialize(Some(types::Capabilities {
                supports_configuration_done_request: Some(true),
                supports_evaluate_for_hovers: Some(true),
                supports_function_breakpoints: Some(true),
                ..Default::default()
            })),
        )
    } else {
        Response::make_error(&request, "Missing client name")
    };
    stdout
        .write(Sendable::Response(response))
        .expect("failed to write to stdout");
}

/// Handle the `launch` request. At this point, our server will:
/// - start gdb -> but the debuggee will not be executed yet
/// - send process event
/// - send initialized event
pub async fn handle_launch(
    request: Request,
    driver: &mut DriverGDB,
    stdout: &mut StdoutWriter,
) -> Receiver<Record> {
    let Command::Launch(args) = &request.command else {
        panic!("expected Command::Launch!");
    };

    let output_channel = match driver.launch(&args).await {
        Ok((process_event, recv)) => {
            stdout
                .write(Sendable::Response(Response::make_success(
                    &request,
                    ResponseBody::Launch,
                )))
                .unwrap();

            // send the ProcessEvent as well
            stdout
                .write(Sendable::Event(Event::make_event(EventBody::Process(
                    process_event,
                ))))
                .unwrap();

            // and send the initialized event
            stdout
                .write(Sendable::Event(Event::make_event(EventBody::Initialized)))
                .expect("failed to write to stdout");
            recv
        }
        Err(e) => {
            stdout
                .write(Sendable::Response(Response::make_error(
                    &request,
                    &format!("Failed to launch gdb. {}", e),
                )))
                .expect("failed to write to stdout");
            panic!("failed to launch gdb with arguments: {:?}", args);
        }
    };
    output_channel
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

    #[allow(dead_code)]
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

            let mut stdout = StdoutWriter::new();
            handle_initialise(request, &mut stdout).await;
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
            let mut stdout = StdoutWriter::new();
            let mut driver_gdb = DriverGDB::new();
            handle_launch(request, &mut driver_gdb, &mut stdout).await;
        })
    }
}
