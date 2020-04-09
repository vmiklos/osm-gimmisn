/*
 * Copyright 2020 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

/* global Chart */

function addCharts(stats) {
    var daily = stats.daily;
    var dailytotal = stats.dailytotal;
    var monthly = stats.monthly;
    var monthlytotal = stats.monthlytotal;
    var topusers = stats.topusers;
    var progress = stats.progress;

    var dailyData = {
        // daily is a list of label-data pairs.
        labels: daily.map(x => x[0]),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: daily.map(x => x[1]),
        }]
    };
    var dailyCtx = document.getElementById("daily").getContext("2d");
    new Chart(dailyCtx, {
        type: "bar",
        data: dailyData,
        options: {
            title: {
                display: true,
                text: "New house numbers, last 2 weeks, as of " + progress.date,
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: "Time"
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "New house numbers"
                    },
                }]
            },
            plugins: {
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            tooltips: {
                enabled: false,
            },
            legend: {
                display: false,
            },
        }
    });

    var monthlyData = {
        // monthly is a list of label-data pairs.
        labels: monthly.map(x => x[0]),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthly.map(x => x[1]),
        }]
    };
    var monthlyCtx = document.getElementById("monthly").getContext("2d");
    new Chart(monthlyCtx, {
        type: "bar",
        data: monthlyData,
        options: {
            title: {
                display: true,
                text: "New house numbers, last year, as of " + progress.date,
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: "Time"
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "New house numbers"
                    },
                }]
            },
            plugins: {
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            tooltips: {
                enabled: false,
            },
            legend: {
                display: false,
            },
        }
    });

    var monthlytotalData = {
        // monthlytotal is a list of label-data pairs.
        labels: monthlytotal.map(x => x[0]),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthlytotal.map(x => x[1]),
        }]
    };
    var monthlyTotalCtx = document.getElementById("monthlytotal").getContext("2d");
    new Chart(monthlyTotalCtx, {
        type: "line",
        data: monthlytotalData,
        options: {
            title: {
                display: true,
                text: "All house numbers, last year, as of " + progress.date,
            },
            scales: {
                xAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "Time"
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "All house numbers"
                    },
                }]
            },
            plugins: {
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            tooltips: {
                enabled: false,
            },
            legend: {
                display: false,
            },
        }
    });

    var dailytotalData = {
        // dailytotal is a list of label-data pairs.
        labels: dailytotal.map(x => x[0]),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: dailytotal.map(x => x[1]),
        }]
    };

    var dailyTotalCtx = document.getElementById("dailytotal").getContext("2d");
    new Chart(dailyTotalCtx, {
        type: "line",
        data: dailytotalData,
        options: {
            title: {
                display: true,
                text: "All house numbers, last 2 weeks, as of " + progress.date,
            },
            scales: {
                xAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "Time"
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "All house numbers"
                    },
                }]
            },
            plugins: {
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            tooltips: {
                enabled: false,
            },
            legend: {
                display: false,
            },
        }
    });

    var topusersData = {
        // topusers is a list of label-data pairs.
        labels: topusers.map(x => x[0]),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: topusers.map(x => x[1]),
        }]

    };
    var topUsersCtx = document.getElementById("topusers").getContext("2d");
    new Chart(topUsersCtx, {
        type: "bar",
        data: topusersData,
        options: {
            title: {
                display: true,
                text: "Top house number editors, as of " + progress.date,
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: "User name"
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "Number of house numbers last changed by this user",
                    },
                }]
            },
            plugins: {
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            tooltips: {
                enabled: false,
            },
            legend: {
                display: false,
            },
        }
    });

    var progressData = {
        datasets: [{
            label: "Reference",
            backgroundColor: "rgba(255, 0, 0, 0.5)",
            data: [ progress.reference ],
        }, {
            label: "OSM",
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: [ progress.osm ],
        }]

    };
    var progressCtx = document.getElementById("progress").getContext("2d");
    new Chart(progressCtx, {
        type: "horizontalBar",
        data: progressData,
        options: {
            title: {
                display: true,
                text: "Coverage is " + progress.percentage + "%, as of " + progress.date,
            },
            scales: {
                xAxes: [{
                    ticks: { min: 0.0, },
                    scaleLabel: {
                        display: true,
                        labelString: "Number of house numbers in database",
                    },
                }]
            },
            plugins: {
                datalabels: {
                    // eslint-disable-next-line no-unused-vars
                    formatter: function(value, context) {
                        // Turn 1000 into '1 000'.
                        return value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, " ");
                    }
                }
            },
            tooltips: {
                enabled: false,
            }
        }
    });
}

// eslint-disable-next-line no-unused-vars
document.addEventListener("DOMContentLoaded", function(event) {
    // This could be configurable, but currently it's the only valid value.
    var statsJSON = "https://osm.vmiklos.hu/osm/housenumber-stats/hungary/stats.json";
    window.fetch(statsJSON)
        .then((response) => {
            return response.json();
        })
        .then((stats) => {
            addCharts(stats);
        });
});

// vim: shiftwidth=4 softtabstop=4 expandtab:
