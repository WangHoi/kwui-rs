import { Theme } from "./Theme.js";

globalThis.showConfirmDialog = function (props) {
    //console.log("showConfirmDialog:", JSON.stringify(data));
    let dialogId = app.showDialog({
        title: props.title,
        width: Theme.CONFIRM_DIALOG_WIDTH,
        height: Theme.CONFIRM_DIALOG_HEIGHT,
        flags: 1,
        customFrame: {
            image: Theme.DIALOG_SHADOW_PNG,
            padding: Theme.DIALOG_SHADOW_MARGIN_PIXELS,
        },
        modulePath: "./ConfirmDialog.js",
        moduleParams: props,
    });
    return dialogId;
}

globalThis.closeDialog = function (id) {
    app.closeDialog(id);
}

globalThis.getDialogHwnd = function (id) {
    app.getDialogHwnd(id);
}

globalThis.getDialogDpiScale = function (id) {
    app.getDialogDpiScale(id);
}

globalThis.showInstallDialog = function (props) {
    //console.log("showConfirmDialog:", JSON.stringify(data));
    let dialogId = app.showDialog({
        title: props.displayName + "安装向导",
        width: 552,
        height: 408,
        flags: 1,
        customFrame: {
            image: Theme.DIALOG_SHADOW_PNG,
            padding: Theme.DIALOG_SHADOW_MARGIN_PIXELS,
        },
        modulePath: "./InstallDialog.js",
        moduleParams: props,
    });
    return dialogId;
}
