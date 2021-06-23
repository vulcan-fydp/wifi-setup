$(document).ready(function(){
    $("#connect").click(function() {
        $.post("switch_connect", function(data) {});
    });

    $("#press_a").click(function() {
        $.post("press_a", function(data) {});
    });
});
