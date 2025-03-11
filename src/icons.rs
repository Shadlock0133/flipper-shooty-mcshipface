use flipperzero_sys as sys;

#[repr(transparent)]
struct Frame(*const u8);
unsafe impl Sync for Frame {}

#[repr(transparent)]
struct Frames(*const *const u8);
unsafe impl Sync for Frames {}

macro_rules! icon {
    ( $width: literal, $height: literal, $name: literal ) => {{
        sys::Icon {
            width: $width,
            height: $height,
            frame_count: 1,
            frame_rate: 0,
            frames: {
                static FRAME: Frames = {
                    static DATA: &[u8] = include_bytes!(concat!(
                        env!("OUT_DIR"),
                        "/",
                        $name,
                        ".icon"
                    ));
                    static DATA_PTR: &[Frame] = &[Frame(DATA.as_ptr())];
                    Frames(DATA_PTR.as_ptr().cast())
                };
                FRAME.0
            },
        }
    }};
}

include!(concat!(env!("OUT_DIR"), "/icons.rs"));
