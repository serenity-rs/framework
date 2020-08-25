use serenity::model::channel::Message;

use serenity_standard_framework::prelude::*;
use serenity_standard_framework::DefaultError;

#[derive(Default)]
struct TestData {
    text: String,
}

fn _ping(ctx: FrameworkContext<TestData>, _msg: Message) -> BoxFuture<'static, CommandResult> {
    Box::pin(async move {
        println!("{:?}", ctx.data.read().await.text);

        Ok(())
    })
}

fn ping() -> Command<TestData> {
    Command::builder("ping").function(_ping).build()
}

fn general() -> Group {
    Group::builder("general").command(ping).build()
}

#[tokio::test]
async fn construction() {
    let _framework: Framework = Framework::new(Configuration::new());
    let _framework: Framework<(), DefaultError> = Framework::new(Configuration::new());
    let _framework: Framework<TestData> = Framework::new(Configuration::new());

    let mut conf = Configuration::new();
    conf.group(general);

    let _framework: Framework<TestData> = Framework::with_data(
        conf,
        TestData {
            text: "42 is the answer to life, the universe, and everything.".to_string(),
        },
    );
}
