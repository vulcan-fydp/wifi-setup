function querySSIDs() {
    $.ajax({
        url: 'scan_ssids',
        success: function(data) {
            $('.ssid-list').html('');
            data['ssids'].forEach(ssid =>
                $('.ssid-list').append('<a href="/ssid/' + ssid + '">' + ssid + '</a>')
            );
        }
    });
}

$(document).ready(function() {
    setInterval(querySSIDs, 5000);
});
