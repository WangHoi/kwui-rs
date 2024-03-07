import { useState } from "Keact";
import { useNativeProp } from "./util.js";
import { Theme } from "./Theme.js"

function ItemEntry({ title, content }) {
    let [expanded, setExpanded] = useState(false);
    let btn_text = expanded ? "收起" : "展开";
    return (
        <div style="margin: 4px 0px;">
            <p>
                <span class={expanded ? "title expanded" : "title"}>{title}</span><button class="small" onclick={() => setExpanded(!expanded)}>{btn_text}</button>
            </p>
            {expanded ? <p style="font-size:14px;">{content}</p> : <p></p>}
        </div>
    );

}

export function MainDialog(props, kids) {
    let channel = useNativeProp(getChannel, "main-dialog:channel-loaded");
    let text = channel.items.length == 0 ? "没有数据" : "已加载：";
    return (
        <body>
            <div>
                <button class="primary" style="margin-right: 8px;" onclick={() => reloadChannel()}>加载</button>
                <span style="font-size:24px; line-height: 32px;">{`${text}${channel.title}`}</span>
            </div>
            <div>
                {
                    channel.items.map(item => <ItemEntry title={item.title} content={item.description} />)
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
kml {
	height: 100%;
	overflow: auto;
}
body {
    margin: 16px;
}
button.primary {
	padding: 8px 8px;
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
button.small {
    margin-left: 4px;
	padding: 4px 4px;
	border: 1px solid black;
    font-size: ${Theme.H3_FONT_SIZE};
    color: black;
}
button.small:hover {
	background-color: lightcyan;
}
button.small:active {
	background-color: lightblue;
}
.title {
    font-size: ${Theme.H3_FONT_SIZE};
    font-weight: bold;
}
.expanded {
    color: blue;
    font-style: italic;
}
`;

export function builder() {
    return {
        root: <MainDialog />,
        stylesheet: MainDialogStyle,
    };
}
