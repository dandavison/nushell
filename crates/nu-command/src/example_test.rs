#[cfg(test)]
use nu_engine;
#[cfg(test)]
use nu_parser;
#[cfg(test)]
use nu_protocol::{
    ast::Block,
    engine::{Command, EngineState, Stack, StateDelta, StateWorkingSet},
    PipelineData, Span, SyntaxShape, Value,
};
#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
use crate::To;

#[cfg(test)]
use super::{
    Ansi, Date, From, If, Into, LetEnv, Math, Path, Random, Split, SplitColumn, SplitRow, Str,
    StrJoin, StrLength, StrReplace, Url, Wrap,
};

#[cfg(test)]
pub fn test_examples(cmd: impl Command + 'static) {
    use crate::BuildString;

    let examples = cmd.examples();
    let signature_output_shape = cmd.signature().output_shape;
    let mut engine_state = Box::new(EngineState::new());

    let delta = {
        // Base functions that are needed for testing
        // Try to keep this working set small to keep tests running as fast as possible
        let mut working_set = StateWorkingSet::new(&*engine_state);
        working_set.add_decl(Box::new(Str));
        working_set.add_decl(Box::new(StrJoin));
        working_set.add_decl(Box::new(StrLength));
        working_set.add_decl(Box::new(StrReplace));
        working_set.add_decl(Box::new(BuildString));
        working_set.add_decl(Box::new(From));
        working_set.add_decl(Box::new(If));
        working_set.add_decl(Box::new(To));
        working_set.add_decl(Box::new(Into));
        working_set.add_decl(Box::new(Random));
        working_set.add_decl(Box::new(Split));
        working_set.add_decl(Box::new(SplitColumn));
        working_set.add_decl(Box::new(SplitRow));
        working_set.add_decl(Box::new(Math));
        working_set.add_decl(Box::new(Path));
        working_set.add_decl(Box::new(Date));
        working_set.add_decl(Box::new(Url));
        working_set.add_decl(Box::new(Ansi));
        working_set.add_decl(Box::new(Wrap));
        working_set.add_decl(Box::new(LetEnv));

        use super::Echo;
        working_set.add_decl(Box::new(Echo));
        // Adding the command that is being tested to the working set
        working_set.add_decl(Box::new(cmd));

        working_set.render()
    };

    let cwd = std::env::current_dir().expect("Could not get current working directory.");

    engine_state
        .merge_delta(delta)
        .expect("Error merging delta");

    for example in examples {
        // Skip tests that don't have results to compare to
        if example.result.is_none() {
            continue;
        }
        // I will move everything inside the pattern match to avoid the unwrap but leaving
        // it like this for now to keep the diff clear.
        let expected_result = example.result.as_ref().unwrap();
        match signature_output_shape {
            SyntaxShape::Any => {
                // Any is the default; remove this branch when output_shape declarations have
                // been added for all commands.
            }
            // TODO: Examples using column paths will fail this test, since the output type is often
            // different when column paths are used. Detect such examples and handle appropriately.

            // TODO: Introduce a rule to nushell stating that flags and positional arguments may not
            // alter the output type. So, for example, `first` returns the first item but we will
            // get rid of `first n` since that changes the return type, and the functionality is
            // already available under the name `take`. Another example of something that would not
            // be allowed is `transpose -d`.
            _ => assert_eq!(
                expected_result.get_type().to_shape(),
                signature_output_shape,
                "Example result type does not match declared command output type"
            ),
            // TODO: The above works for output; but we want to test that the Example input matches
            // the declared input type also. Obtain the input syntax shape from the Example and
            // check that it matches the declared Signature.input_shape. This may involve
            // refactoring the way that Examples are defined so that the input command is available
            // as a separate field?
        }
        let start = std::time::Instant::now();

        let mut stack = Stack::new();

        // Set up PWD
        stack.add_env_var(
            "PWD".to_string(),
            Value::String {
                val: cwd.to_string_lossy().to_string(),
                span: Span::test_data(),
            },
        );

        engine_state
            .merge_env(&mut stack, &cwd)
            .expect("Error merging environment");

        let empty_input = PipelineData::new(Span::test_data());
        let result = eval(example.example, empty_input, &cwd, &mut engine_state);

        println!("input: {}", example.example);
        println!("result: {:?}", result);
        println!("done: {:?}", start.elapsed());

        // Note. Value implements PartialEq for Bool, Int, Float, String and Block
        // If the command you are testing requires to compare another case, then
        // you need to define its equality in the Value struct
        if let Some(expected) = example.result {
            if result != expected {
                panic!(
                    "the example result is different to expected value: {:?} != {:?}",
                    result, expected
                )
            }
        }
    }
}

#[cfg(test)]
fn eval(
    contents: &str,
    input: PipelineData,
    cwd: &PathBuf,
    engine_state: &mut Box<EngineState>,
) -> Value {
    let (block, delta) = parse(contents, engine_state);
    eval_block(block, input, cwd, engine_state, delta)
}

#[cfg(test)]
fn parse(contents: &str, engine_state: &Box<EngineState>) -> (Block, StateDelta) {
    let mut working_set = StateWorkingSet::new(&*engine_state);
    let (output, err) = nu_parser::parse(&mut working_set, None, contents.as_bytes(), false, &[]);

    if let Some(err) = err {
        panic!("test parse error in `{}`: {:?}", contents, err)
    }

    (output, working_set.render())
}

#[cfg(test)]
fn eval_block(
    block: Block,
    input: PipelineData,
    cwd: &PathBuf,
    engine_state: &mut Box<EngineState>,
    delta: StateDelta,
) -> Value {
    engine_state
        .merge_delta(delta)
        .expect("Error merging delta");

    let mut stack = Stack::new();

    stack.add_env_var(
        "PWD".to_string(),
        Value::String {
            val: cwd.to_string_lossy().to_string(),
            span: Span::test_data(),
        },
    );

    match nu_engine::eval_block(&engine_state, &mut stack, &block, input, true, true) {
        Err(err) => panic!("test eval error in `{}`: {:?}", "TODO", err),
        Ok(result) => result.into_value(Span::test_data()),
    }
}
