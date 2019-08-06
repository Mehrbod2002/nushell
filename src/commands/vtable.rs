use crate::commands::StaticCommand;
use crate::errors::ShellError;
use crate::format::VTableView;
use crate::prelude::*;

pub struct VTable;

#[derive(Deserialize)]
pub struct VTableArgs {}

impl StaticCommand for VTable {
    fn name(&self) -> &str {
        "vtable"
    }
    fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        args.process(registry, vtable)?.run()
    }
    fn signature(&self) -> Signature {
        Signature::build("vtable")
    }
}

pub fn vtable(_args: VTableArgs, context: RunnableContext) -> Result<OutputStream, ShellError> {
    let stream = async_stream_block! {
        let input = context.input.into_vec().await;

        if input.len() > 0 {
            let mut host = context.host.lock().unwrap();
            let view = VTableView::from_list(&input);
            if let Some(view) = view {
                handle_unexpected(&mut *host, |host| crate::format::print_view(&view, host));
            }
        }
    };

    Ok(OutputStream::new(stream))
}
