function Hello() {
	return <p>Hello kwui</p>;
}

var helloStyle = css`
p {
	margin-top: 16px;
	text-align: center;
	color: orange;
	font-weight: bold;
	font-size: 24px;
}
`;

export function builder() {
	return {
		root: <Hello />,
		stylesheet: helloStyle,
	}
}