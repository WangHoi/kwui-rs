function Hello() {
	return <p>Hello kwui</p>;
}

var helloStyle = css`
p {
	color: orangered;
}
`;

export function builder() {
	return {
		root: <Hello />,
		stylesheet: helloStyle,
	}
}