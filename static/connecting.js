function queryConnected() {
    $.ajax({
        url: 'is_connected',
        success: function(data) {
            if (data['connected']) {
                $('h1').html('Connected!');
            }
        }
    });
}

$(document).ready(function() {
    setInterval(queryConnected, 1000);
});
