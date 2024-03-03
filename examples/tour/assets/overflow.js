var simple_stylesheet = css`
kml {
	height: 100%;
	overflow: auto;
}
.outer {
	display: block;
	overflow: auto;
	width: 176px;
	height: 176px;
	margin: 0px;
	padding: 10px 10px;
	background-color: #fff;
	border-width: 2px;
	border-color: #000;
}
.overflow-auto {
	overflow: auto;
}
.inner {
	width: 200px;
	height: 100px;
	background-color: #00ffff;
	font-size: 16px;
	border-width: 2px;
	border-color: #080;
	margin-left: 50px;
}
span {
	font-size: 16;
}
line_edit {
	width: 100;
	height: 20;
	background-color: #fff;
}
.inline-block {
	display: inline-block;
}
button {
	display: inline-block;
	border: 1px solid black;
	padding: 0px 4px;
	height: 24px;
	line-height: 24px;
}
button:hover {
	background-color: lightblue;
}
button:active {
	background-color: lightcyan;
}
`;

var span1 = <span>This property is a shorthand for the following CSS properties: border-top-left-radius border-top-right-radius border-bottom-right-radius border-bottom-left-radius.</span>;

function VerticalOverflow() {
	return <div class="outer">
		{"Vertical overflow"}
		<p><span style="text-decoration: underline; font-weight:bold;font-size:20px;">border-radius</span></p>
		<p>{span1}</p>
		<button>Test</button>
		<p>{span1}</p>
	</div>;
}
function HorizontalOverflow() {
	return <div class="outer">
		{"Horizontal overflow"}
		<div class="inner" />
		<button>Test</button>
	</div>;
}
function TwoDirectionOverflow() {
	return <div class="outer">
		{"Two direction overflow"}
		<div class="inner" />
		{span1}
		<button>Test</button>
		<div class="inner" />
	</div>;
}
function NestedOverflow() {
	return <div class="outer" style="width:600px;height:320px;">
		{"Nested overflow"}
		<div class="inner" />
		{span1}
		<button>Test</button>
		<div class="outer">
			{"Inner overflow"}
			<div class="inner" />
			{span1}
			<button>Test</button>
		</div>
		<div class="inner" />
	</div>;
}
function OverflowExample() {
	return <div style="margin: 16px;">
		{[
			// <button>Test</button>,
			<div class="inline-block"><VerticalOverflow /></div>,
			<div class="inline-block" style="width: 8px;"></div>,
			<div class="inline-block"><HorizontalOverflow /></div>,
			<div class="inline-block" style="width: 8px;"></div>,
			<div class="inline-block"><TwoDirectionOverflow /></div>,
			<NestedOverflow />
		]}
	</div>;
}

var box_overflow = (<div class="outer">
	<div class="inner" />
	<div class="inner" />
	<div class="inner" />
</div>);

export function builder() {
	return {
		root: <OverflowExample />,
		stylesheet: simple_stylesheet,
	}
}