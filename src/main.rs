use std::time::{Duration, Instant};

use iced::{
    button, executor, Align, Application, Button, Column, Command, Element, Font,
    HorizontalAlignment, Length, Row, Settings, Subscription, Text,
};
use iced_futures::futures;
use iced_native::Color;

const FPS: u64 = 30;
const MILLISEC: u64 = 1000;
const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;

// 外部からダウンロードしてきたフォントファイル(.ttf)を適用
const FONT: Font = Font::External {
    name: "PixelMplus12-Regular",
    bytes: include_bytes!("../rsc/PixelMplus12-Regular.ttf"),
};

// 今回のアプリケーションを司る構造体
struct GUI {
    tick_state: TickState,
    start_stop_button_state: button::State,
    reset_button_state: button::State,
    last_update: Instant,
    total_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,  // 時間の測定を開始するメッセージ
    Stop,   // 時間の測定を停止するメッセージ
    Reset,  // 測定した時間をリセットするメッセージ
    Update, // 測定した時間を更新するメッセージ
}

// 測定中か否かを管理するための条件
pub enum TickState {
    Init,
    Stopped,
    Ticking,
}

pub struct Timer {
    duration: Duration,
}

impl Timer {
    fn new(duration: Duration) -> Timer {
        Timer {duration}
    }
}

impl<H, E> iced_native::subscription::Recipe<H, E> for Timer where H: std::hash::Hasher {
    // Streamから出力される型
    type Output = Instant;

    // それぞれのSubscriptionをハッシュで比較できるようにするためのメソッド
    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        // ハッシュ計算用の値はなんでもよいので、今回の計測するdurationをセットしている
        std::any::TypeId::of::<Self>().hash(state);
        self.duration.hash(state)
    }

    // Recipeを実行し、Subscriptionのイベントを出力するStreamを作り出すためのメソッド
    fn stream(self: Box<Self>, _input: futures::stream::BoxStream<E>)
        -> futures::stream::BoxStream<Self::Output> {
            use futures::stream::StreamExt;

            // 一定間隔で現在時刻を返す
            async_std::stream::interval(self.duration)
                .map(|_| Instant::now())
                .boxed()
    }
}

// 構造体GUIにApplicationトレイトを実装
impl Application for GUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    // new runした際に、icedの内部で使われる初期化のためのメソッド
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            GUI {
                tick_state: TickState::Init,
                start_stop_button_state: button::State::new(),
                reset_button_state: button::State::new(),
                last_update: Instant::now(),
                total_duration: Duration::default(),
            },
            Command::none(),
        )
    }

    // title ウィンドウのタイトル
    fn title(&self) -> String {
        String::from("DEMO")
    }

    // update ランタイムシステムからメッセージを受け取り、そのメッセージによってアプリケーションの状態を
    // 更新するメソッド
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Start => {
                // Startボタン押下時、状態をTickingに切り替え、最終更新時刻に現在時刻をセット
                self.tick_state = TickState::Ticking;
                self.last_update = Instant::now();
            },

            Message::Stop => {
                // Stopボタン押下時、状態をStoppedに切り替え、累計経過時間に現在時刻と最終更新時刻の差分をセット
                self.tick_state = TickState::Stopped;
                self.total_duration += Instant::now() - self.last_update;
            },

            Message::Reset => {
                // Resetボタン押下時、最終更新時刻・累計経過時間をリセット
                self.last_update = Instant::now();
                self.total_duration = Duration::default();
                self.tick_state = TickState::Init;
            },

            Message::Update => match self.tick_state {
                // 時間計測時、状態がTickingの場合のみ、
                // 累計経過時間を現在時刻と最終更新時刻の差分をセットした後、最終更新時刻を現在時刻に更新
                TickState::Ticking => {
                    let now_update = Instant::now();
                    self.total_duration += now_update - self.last_update;
                    self.last_update = now_update;
                },
                _ => {}
            },
        }

        Command::none()
    }

    // view ウィンドウに表示するウィジェットを設定するためのメソッド
    fn view(&mut self) -> Element<'_, Self::Message> {
        let seconds = self.total_duration.as_secs();

        // display texts
        let duration_text = format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            seconds / HOUR,
            (seconds % HOUR) / MINUTE,
            seconds % HOUR,
            self.total_duration.subsec_millis() / 10
        );

        let start_stop_text = match self.tick_state {
            TickState::Init => Text::new("Start")
                .horizontal_alignment(HorizontalAlignment::Center)
                .font(FONT),
            TickState::Stopped => Text::new("Restart")
                .horizontal_alignment(HorizontalAlignment::Center)
                .font(FONT),
            TickState::Ticking => Text::new("Stop")
                .horizontal_alignment(HorizontalAlignment::Center)
                .font(FONT),
        };

        let start_stop_message = match self.tick_state {
            TickState::Init | TickState::Stopped => Message::Start,
            TickState::Ticking => Message::Stop,
        };

        // Base widgets
        let tick_text = Text::new(duration_text).font(FONT).size(60);

        let start_stop_button = Button::new(
            &mut self.start_stop_button_state, start_stop_text
        )
            .min_width(80)
            .on_press(start_stop_message);

        let reset_button = Button::new(
            &mut self.reset_button_state,
            Text::new("Reset")
                .horizontal_alignment(HorizontalAlignment::Center)
                .font(FONT)
        )
            .min_width(80)
            .on_press(Message::Reset);

        // Layout widgets
        Column::new()
            .push(tick_text)
            .push(Row::new()
                .push(start_stop_button)
                .push(reset_button)
                .spacing(10)
            )
            .spacing(10)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let timer = Timer::new(Duration::from_millis(MILLISEC / FPS));
        iced::Subscription::from_recipe(timer).map(|_| Message::Update)
    }
}

fn main() {
    let mut settings = Settings::default();
    settings.window.size = (400, 120); // ウィンドウサイズを固定

    GUI::run(settings);
}
