import { Theme } from "./Theme.js"
import { TitleBar, TitleBarStyle } from "./TitleBar.js";

export function ConfirmDialog({
    title,
    label,
    action_btn, on_action_btn_click,
    cancel_btn, on_cancel_btn_click,
    skip_btn, on_skip_btn_click,

}, kids) {
    let action_button = action_btn
        ? <button class="action" onclick={() => { app.post("confirm-dialog:action-button-clicked", this.dialogId); }}>{action_btn}</button >
        : null;
    let cancel_button = cancel_btn
        ? <button class="border" onclick={() => { app.post("confirm-dialog:cancel-button-clicked", this.dialogId); }}>{cancel_btn}</button>
        : null;
    let skip_button = skip_btn
        ? <button class="border" onclick={() => { app.post("confirm-dialog:skip-button-clicked", this.dialogId); }}>{skip_btn}</button>
        : null;
    return (
        <body>
            <TitleBar title={title} />
            <div class="label">{label}</div>
            <div class="button-bar">
                {action_button}
                {cancel_button}
                {skip_button}
            </div>
        </body>
    );
}

export var ConfirmDialogStyle = TitleBarStyle + `
.label {
    position: absolute;
    left: 36;
    top: 80;
    width: 288;
    height: 48;
    font-size: ${Theme.H3_FONT_SIZE};
    color: ${Theme.H2_TEXT_COLOR};
    text-align: center;
}

.button-bar {
	position: absolute;
	bottom: 0;
	right: 0;
	margin: 16px;
}
button.action, button.border {
	border-radius: ${Theme.SECONDARY_BORDER_RADIUS};
	margin-left: 16px;
}

button.action {
    line-height: 32px;
    padding: 0px 8px;
    font-size: ${Theme.H3_FONT_SIZE};
    color: ${Theme.ACTION_TEXT_COLOR};
	background-color: ${Theme.ACTION_COLOR};
}
button.action:hover {
	background-color: ${Theme.ACTION_HOVERED_COLOR};
}
button.action:active {
	background-color: ${Theme.ACTION_HOVERED_COLOR};
}

button.border {
    padding: 0px 8px;
    line-height: 28px;
    border-color: ${Theme.BORDER_COLOR};
    border-width: 2px;
    font-size: ${Theme.H3_FONT_SIZE};
    color: ${Theme.H3_TEXT_COLOR};
	background-color: #00000000;
}
button.border:hover {
	background-color: ${Theme.BORDER_COLOR};
}
button.border:active {
	background-color: ${Theme.BORDER_COLOR};
}
`;

export function builder({ title, label, action_btn, cancel_btn, skip_btn }) {
    return {
        root: <ConfirmDialog
            title={title}
            label={label}
            action_btn={action_btn}
            cancel_btn={cancel_btn}
            skip_btn={skip_btn}
        />,
        stylesheet: ConfirmDialogStyle,
    };
}
