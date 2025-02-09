use std::time::Duration;

use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, IntoInterruptiblePipelineData, PipelineData, ShellError, Signature, Value,
};

#[derive(Clone)]
pub struct Ps;

impl Command for Ps {
    fn name(&self) -> &str {
        "ps"
    }

    fn signature(&self) -> Signature {
        Signature::build("ps")
            .desc("View information about system processes.")
            .switch(
                "long",
                "list all available columns for each entry",
                Some('l'),
            )
            .filter()
            .category(Category::System)
    }

    fn usage(&self) -> &str {
        "View information about system processes."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        run_ps(engine_state, call)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "List the system processes",
            example: "ps",
            result: None,
        }]
    }
}

fn run_ps(engine_state: &EngineState, call: &Call) -> Result<PipelineData, ShellError> {
    let mut output = vec![];
    let span = call.head;
    let long = call.has_flag("long");

    for proc in nu_system::collect_proc(Duration::from_millis(100), false) {
        let mut cols = vec![];
        let mut vals = vec![];

        cols.push("pid".to_string());
        vals.push(Value::Int {
            val: proc.pid() as i64,
            span,
        });

        cols.push("name".to_string());
        vals.push(Value::String {
            val: proc.name(),
            span,
        });

        #[cfg(not(windows))]
        {
            // Hide status on Windows until we can find a good way to support it
            cols.push("status".to_string());
            vals.push(Value::String {
                val: proc.status(),
                span,
            });
        }

        cols.push("cpu".to_string());
        vals.push(Value::Float {
            val: proc.cpu_usage(),
            span,
        });

        cols.push("mem".to_string());
        vals.push(Value::Filesize {
            val: proc.mem_size() as i64,
            span,
        });

        cols.push("virtual".to_string());
        vals.push(Value::Filesize {
            val: proc.virtual_size() as i64,
            span,
        });

        if long {
            cols.push("command".to_string());
            vals.push(Value::String {
                val: proc.command(),
                span,
            });
        }

        output.push(Value::Record { cols, vals, span });
    }

    Ok(output
        .into_iter()
        .into_pipeline_data(engine_state.ctrlc.clone()))
}
