import { useNativeProp } from "./util.js";
import { Theme } from "./Theme.js"

export function MainDialog(props, kids) {
    let channel = useNativeProp(getChannel, "main-dialog:channel-loaded");
    let text = channel.items.length == 0 ? "没有数据" : "已加载：";
    return (
        <body>
            <div>
                <button class="primary" onclick={() => reloadChannel()}>加载</button>
            </div>
            <div style="font-size:20px; line-height: 32px;">
                <p style="font-size:24px;font-weight: bold; line-height: 32px;">{`${text}${channel.title}`}</p>
                {
                    channel.items.map(item => <p>{item.title}</p>)
                }
            </div>
        </body >
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

export var MainDialogStyle = css`
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

export function builder() {
    return {
        root: <MainDialog />,
        stylesheet: MainDialogStyle,
    };
}
