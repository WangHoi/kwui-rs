import { useNativeProp } from "./util.js";
import { Theme } from "./Theme.js"
import { TitleBar, TitleBarStyle } from "./TitleBar.js";

const MAIN_DIALOG_WIDTH = 552;
const MAIN_DIALOG_HEIGHT = 408;
const EXPAND_DIALOG_HEIGHT = 480;

function MainPageExpanded(props, kids) {
    let browse = () => {
        app.post("install-dialog:browse-button-clicked", this.dialogId);
    };
    let text_changed = (text) => {
        // console.log("text-changed", text);
        app.post("install-dialog:path-text-changed", text);
    };
    let [targetDir, _] = useNativeProp(getTargetDir, "install-dialog:target-dir-changed");
    let [estimateSizeMB, availSizeGB] = useNativeProp(getFreeSpace, "install-dialog:free-space-changed");
    //console.log("getFreeSpace:", JSON.stringify(getFreeSpace()), estimateSizeMB, availSizeGB);
    // console.log("targetDir", targetDir);
    let free_space_label = availSizeGB
        ? `所需空间: ${estimateSizeMB}MB 可用空间: ${availSizeGB}GB`
        : `所需空间: ${estimateSizeMB}MB`;
    let a = (<div style="margin-top:16px;">
        <div style="text-align: center;">
            <div id="path-edit">
                <line_edit
                    value={targetDir}
                    onchange={text_changed}
                    fontSize={Theme.H3_FONT_SIZE}
                    backgroundColor="#00000000"
                    color={Theme.H3_TEXT_COLOR}
                    caretColor="black"
                    selectionBackgroundColor="#FF650040"
                    innerHPadding={8}
                />
            </div>
            <button id="browse-button" onclick={browse}><span>浏览</span></button>
        </div>
        <p id="free-space-label" style="text-align: center;">
            {free_space_label}
        </p>
    </div>);
    return a;
}

function FlatIconTextButton({ icon, onclick }, kids) {
    // console.log("icon:", icon);
    return <button class="flat-icon-text-button" onclick={onclick}>
        <span id="text">更多选项</span>
        <img id="icon" src={icon}></img>
    </button>
}

function MainPage(props, kids) {
    let on_start_click = () => {
        let [_, valid] = getTargetDir();
        if (valid) {
            app.resizeDialog(this.dialogId, MAIN_DIALOG_WIDTH, MAIN_DIALOG_HEIGHT);
            app.post("install-dialog:start-button-clicked");
        }
    };
    let expanded = useNativeProp(isMainPageExpanded, "install-dialog:main-page-expanded");
    let on_expand_click = () => {
        app.post("install-dialog:expand-button-clicked");
        if (expanded) {
            // collapse
            app.resizeDialog(this.dialogId, MAIN_DIALOG_WIDTH, MAIN_DIALOG_HEIGHT);
        } else {
            app.resizeDialog(this.dialogId, MAIN_DIALOG_WIDTH, EXPAND_DIALOG_HEIGHT);
        }
    };
    let flat_icon = expanded ? ":/images/collapse.png" : ":/images/expand.png";
    // console.log("expanded:", expanded, "flat_icon:", flat_icon);
    return <div>
        <div style="margin-top: 20px; text-align: center;">
            <button class="primary" id="start-button" onclick={on_start_click}>{"安装"}</button>
        </div>
        <div style="text-align: center; margin-top: 16px;">
            <FlatIconTextButton icon={flat_icon} onclick={on_expand_click} />
        </div>
        {expanded ? <MainPageExpanded /> : undefined}
    </div>
}

function ProgressPage(props, kids) {
    let progress = useNativeProp(getInstallProgress, "install-dialog:progress-changed");
    return <div style="margin-top: 20px; text-align: center;">
        <progress_bar style="margin-left: auto; margin-right: auto; width: 360px; height: 4px;"
            value={progress}
            backgroundColor={Theme.BORDER_COLOR}
            color={Theme.ACTION_HOVERED_COLOR}
        />
        <p id="progress-label" style="margin-top: 20px;">{`${Math.round(progress * 100)}%`}</p>
    </div>
}

function DonePage(props, kids) {
    let done_install = () => {
        app.post("install-dialog:done-button-clicked");
    };
    return <div>
        <p id="done-label">安装完成</p>
        <div style="text-align:center;">
            <button class="primary" id="done-button" onclick={done_install}>
                {"开始使用"}
            </button>
        </div>
    </div>
}

function PageStack({ current }) {
    if (current === "done") {
        return <DonePage />;
    } else if (current === "progress") {
        return <ProgressPage />
    } else {
        return <MainPage />;
    }
}

export function InstallDialog({
    displayName,
    version,
}, kids) {
    let current = useNativeProp(getCurrentPage, "install-dialog:current-page-changed");
    let pos = displayName.indexOf("(");
    let mainLabelText = (pos === -1) ? displayName : displayName.substring(0, pos);
    return (
        <body>
            <TitleBar title={displayName + "安装向导"} />
            <div class="logo" />
            <div id="name-version" style="text-align:center">
                <span id="name" style="position:relative;">
                    {mainLabelText}
                    <span id="version" style="position:absolute; left:100%; top:-10; width: 50px; text-align: left;">{version}</span>
                </span>
            </div>
            <PageStack current={current} />
        </body>
    );
}

var FlatIconTextButtonStyle = css`
.flat-icon-text-button #icon {
	display: inline-block;
	width: 24px;
	height: 24px;
    vertical-align: middle;
}
.flat-icon-text-button #text {
	color: #777;
	font-size: 14px;
}
.flat-icon-text-button {
	cursor: pointer;
	margin: 8px;
    margin-left: 32px;
    line-height: 24px;
}
`;

export var InstallDialogStyle = TitleBarStyle + FlatIconTextButtonStyle + css`
.logo {
    margin-left: auto;
    margin-right: auto;
    margin-top: ${Theme.LOGO_Y - 32};
    width: ${Theme.LOGO_WIDTH};
    height: ${Theme.LOGO_HEIGHT};
    background-image: url("${Theme.LOGO_PNG}");
}
#name-version {
    margin-top: 16;
    margin-bottom: 68;
}
#name {
    font-weight: normal;
    font-size: ${Theme.H2_FONT_SIZE};
    color: ${Theme.H1_TEXT_COLOR};
}
#version {
    font-size: ${Theme.H3_FONT_SIZE};
    color: ${Theme.H3_TEXT_COLOR};
}
button.primary {
	padding: 8px 56px;
	border-radius: ${Theme.PRIMARY_BORDER_RADIUS};
    font-size: ${Theme.H2_FONT_SIZE};
    color: ${Theme.ACTION_TEXT_COLOR};
	background-color: ${Theme.ACTION_COLOR};
}
button.primary:hover {
	background-color: ${Theme.ACTION_HOVERED_COLOR};
}
button.primary:active {
	background-color: ${Theme.ACTION_HOVERED_COLOR};
}
#path-edit {
    overflow: hidden;
    display: inline-block;
    width: 442;
    height: 32;
    background-color: white;
    border-radius: 4px;
    margin-right: 8px;
}
#path-edit line_edit {
    display: block;
    position: relative;
    top: 0px;
    width: 100%;
    height: 100%;
}
#browse-button span {
    display: inline-block;
    padding: 0px 0px 0px 0px;
    position: relative;
    top: 6px;
}
#browse-button {
    overflow: hidden;
    height: 32px;
    padding-left: 8px;
    padding-right: 8px;
	border-radius: ${Theme.SECONDARY_BORDER_RADIUS};
    font-size: ${Theme.H3_FONT_SIZE};
    color: ${Theme.ACTION_TEXT_COLOR};
	background-color: ${Theme.ACTION_COLOR};
}
#browse-button:hover {
	background-color: ${Theme.ACTION_HOVERED_COLOR};
}
#browse-button:active {
	background-color: ${Theme.ACTION_HOVERED_COLOR};
}
#free-space-label {
    margin-top: 8px;
    color: ${Theme.H3_TEXT_COLOR};
    font-size: ${Theme.H3_FONT_SIZE};
}
#progress-label {
    color: ${Theme.H3_TEXT_COLOR};
    font-size: 40;
}
#done-label {
    position: absolute;
    left: 8;
    top: 250;
    width: 536;
    height: 24;
    font-size: ${Theme.H3_FONT_SIZE};
    color: ${Theme.H3_TEXT_COLOR};
    text-align: center;
}
#done-button {
    margin-top: 16px;
    padding-left: 16px;
    padding-right: 16px;
}
`;

export function builder({ displayName, version }) {
    return {
        root: <InstallDialog displayName={displayName} version={version} />,
        stylesheet: InstallDialogStyle,
    };
}
