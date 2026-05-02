use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Profile {
    #[serde(default)]
    pub(crate) libs: Vec<Lib>,
    pub(crate) threads: Vec<Thread>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Lib {
    pub(crate) path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Thread {
    pub(crate) samples: Samples,
    #[serde(rename = "stackTable")]
    pub(crate) stack_table: StackTable,
    #[serde(rename = "frameTable")]
    pub(crate) frame_table: FrameTable,
    #[serde(rename = "funcTable")]
    pub(crate) func_table: FuncTable,
    #[serde(rename = "resourceTable", default)]
    pub(crate) resource_table: ResourceTable,
    #[serde(rename = "stringArray", default)]
    pub(crate) string_array: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Samples {
    #[serde(default)]
    pub(crate) stack: Vec<Option<usize>>,
    #[serde(rename = "threadCPUDelta")]
    pub(crate) thread_cpu_delta: Option<Vec<serde_json::Number>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StackTable {
    pub(crate) frame: Vec<usize>,
    #[serde(default)]
    pub(crate) prefix: Vec<Option<usize>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FrameTable {
    pub(crate) func: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FuncTable {
    pub(crate) name: Vec<usize>,
    pub(crate) resource: Option<Vec<Option<isize>>>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ResourceTable {
    #[serde(default)]
    pub(crate) lib: Vec<Option<isize>>,
}
