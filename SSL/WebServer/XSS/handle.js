function loadContent() {
	fetch('/XSS/input')
		.then(response => response.text())
		.then(data => {
			document.getElementById('content').innerHTML = data;
		}
	);
}
loadContent();
document.getElementById('safe-submit').addEventListener('click', function(event) {
	event.preventDefault(); // 阻止表单默认提交行为
	const data = document.getElementById('inputText').value;
	fetch('/XSS/safe', {
		method: 'POST',
		headers: {
			'Content-Type': 'text/plain'
		},
		body: data
	})
	.then(_ => {
		loadContent();
	});
});
document.getElementById('unsafe-submit').addEventListener('click', function(event) {
	event.preventDefault(); // 阻止表单默认提交行为
	const data = document.getElementById('inputText').value;
	fetch('/XSS/unsafe', {
		method: 'POST',
		headers: {
			'Content-Type': 'text/plain'
		},
		body: data
	})
	.then(_ => {
		loadContent();
	});
});