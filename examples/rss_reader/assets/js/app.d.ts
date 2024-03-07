declare namespace app {
  type CustomFrameData = {
    image: String,
    padding: number,
  };
  type DialogId = String;
  function showDialog(data: {
    width?: number,
    height?: number,
    flags?: number,
    root: object,
    customFrame?: CustomFrameData,
    stylesheet: String | object,
  }): DialogId;
  function closeDialog(id: DialogId): void;
  function closingDialog(id: DialogId): void;
  function resizeDialog(id: DialogId, width: number, height: number): void;

  type EventCallback = function(String, any): void;
  function post(event: String, arg: any): void;
  function addListener(event: String, cb: EventCallback): void;
  function removeListener(event: String, cb: EventCallback): boolean;
}

type HookState = any;
type HookAction = any;
type ComponentUpdateCallback = function (HookAction): void;
type HookInitCallback = function(ComponentUpdateCallback): HookState;
type HookUpdateCallback = function(HookState, HookAction): [HookState, boolean];
type HookCleanupCallback = function(HookState): void;
function useHook(init: HookInitCallback, update: HookUpdateCallback, cleanup: HookCleanupCallback)
  : [HookState, ComponentUpdateCallback];