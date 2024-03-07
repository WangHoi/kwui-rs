import { Theme } from "./Theme.js"

export function TitleBar({
    title,
}, kids) {
    let did = this.dialogId;
    let onclose = () => app.closingDialog(did);
    return (
        <div id="title-bar">
            <p id="title-bar-label">{title}</p>
            <button id="close-dialog-button" onclick={onclose} />
        </div>
    );
}

export var TitleBarStyle = `
#title-bar {
	position: absolute;
	top: 0;
	left: 0;
    width: 100%;
    height: 32;
}
#title-bar-label {
    position: absolute;
    font-size: 14px;
    color: ${Theme.H3_TEXT_COLOR};
    left: 16;
    top: 8;
}
button#close-dialog-button {
    position: absolute;
    right: 0;
    width: 40px;
    height: 32px;
    background-image: url(":/images/close_button.png");
}
button#close-dialog-button:hover {
    background-color: ${Theme.CLOSE_BUTTON_HOVER_COLOR};
    background-image: url(":/images/close_button_hover.png");
}
`;