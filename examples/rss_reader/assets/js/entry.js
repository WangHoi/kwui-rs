import { Theme } from "./Theme.js";

globalThis.closeDialog = function (id) {
    app.closeDialog(id);
}

globalThis.getDialogHwnd = function (id) {
    app.getDialogHwnd(id);
}

globalThis.getDialogDpiScale = function (id) {
    app.getDialogDpiScale(id);
}

globalThis.showMainDialog = function (props) {
    //console.log("showConfirmDialog:", JSON.stringify(data));
    let dialogId = app.showDialog({
        title: "RSS Reader",
        width: 640,
        height: 480,
        flags: 1,
        modulePath: "./MainDialog.js",
        moduleParams: props,
    });
    return dialogId;
}

showMainDialog({});