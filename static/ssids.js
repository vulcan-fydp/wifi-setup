function querySSIDs() {
    $.ajax({
        url: 'scan_ssids',
        success: function(data) {
            $('.ssid-list').innerHTML = '';
            data['ssids'].forEach(ssid =>
                $('.ssid-list').append('<a href="/ssid/' + ssid + '">' + ssid + '<=a>');
            );
        }
    });
}

$(document).ready(function() {
    setInterval(queryConnected, 5000);
});
