/*
 * Copyright 2020 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

var Chart = require("chart.js");
// eslint-disable-next-line no-unused-vars
var chartJsDatalabels = require("chartjs-plugin-datalabels"); // only for its side-effects
var chartJsTrendline = require("chartjs-plugin-trendline");
Chart.plugins.register(chartJsTrendline);

var config = require("./config.js");

function getString(key: string) {
    return document.getElementById(key).getAttribute("data-value");
}

function addCharts(stats: any) {
    var daily = stats.daily;
    var dailytotal = stats.dailytotal;
    var monthly = stats.monthly;
    var monthlytotal = stats.monthlytotal;
    var topusers = stats.topusers;
    var topcities = stats.topcities;
    var usertotal = stats.usertotal;
    var progress = stats.progress;
    var trendlineOptions = {
        style: "rgba(255,105,180, .8)",
        lineStyle: "dotted",
        width: 2,
    };

    var dailyData = {
        // daily is a list of label-data pairs.
        labels: daily.map(function(x: any[]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: daily.map(function(x: any[]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    let dailyCanvas = <HTMLCanvasElement>document.getElementById("daily");
    var dailyCtx = dailyCanvas.getContext("2d");
    new Chart(dailyCtx, {
        type: "bar",
        data: dailyData,
        options: {
            title: {
                display: true,
                text: getString("str-daily-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-daily-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-daily-y-axis"),
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
        labels: monthly.map(function(x: any[]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthly.map(function(x: any[]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    let monthlyCanvas = <HTMLCanvasElement>document.getElementById("monthly");
    var monthlyCtx = monthlyCanvas.getContext("2d");
    new Chart(monthlyCtx, {
        type: "bar",
        data: monthlyData,
        options: {
            title: {
                display: true,
                text: getString("str-monthly-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-monthly-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-monthly-y-axis"),
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
        labels: monthlytotal.map(function(x: any[]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthlytotal.map(function(x: any[]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    let monthlyTotalCanvas = <HTMLCanvasElement>document.getElementById("monthlytotal");
    var monthlyTotalCtx = monthlyTotalCanvas.getContext("2d");
    new Chart(monthlyTotalCtx, {
        type: "line",
        data: monthlytotalData,
        options: {
            title: {
                display: true,
                text: getString("str-monthlytotal-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-monthlytotal-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-monthlytotal-y-axis"),
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
        labels: dailytotal.map(function(x: any[]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: dailytotal.map(function(x: any[]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };

    let dailyTotalCanvas = <HTMLCanvasElement>document.getElementById("dailytotal");
    var dailyTotalCtx = dailyTotalCanvas.getContext("2d");
    new Chart(dailyTotalCtx, {
        type: "line",
        data: dailytotalData,
        options: {
            title: {
                display: true,
                text: getString("str-dailytotal-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-dailytotal-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-dailytotal-y-axis"),
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
        labels: topusers.map(function(x: any[]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: topusers.map(function(x: any[]) { return x[1]; }),
        }]

    };
    let topUsersCanvas = <HTMLCanvasElement>document.getElementById("topusers");
    var topUsersCtx = topUsersCanvas.getContext("2d");
    new Chart(topUsersCtx, {
        type: "bar",
        data: topusersData,
        options: {
            title: {
                display: true,
                text: getString("str-topusers-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-topusers-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-topusers-y-axis"),
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
    var topcitiesData = {
        // topcities is a list of label-data pairs.
        labels: topcities.map(function(x: any[]) {
            if (x[0] === "_Empty") {
                return getString("str-topcities-empty");
            }
            if (x[0] === "_Invalid") {
                return getString("str-topcities-invalid");
            }
            return x[0];
        }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: topcities.map(function(x: any[]) { return x[1]; }),
        }]

    };
    let topCitiesCanvas = <HTMLCanvasElement>document.getElementById("topcities");
    var topCitiesCtx = topCitiesCanvas.getContext("2d");
    new Chart(topCitiesCtx, {
        type: "bar",
        data: topcitiesData,
        options: {
            title: {
                display: true,
                text: getString("str-topcities-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-topcities-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-topcities-y-axis"),
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

    var usertotalData = {
        // usertotal is a list of label-data pairs.
        labels: usertotal.map(function(x: any[]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: usertotal.map(function(x: any[]) { return x[1]; }),
        }]

    };
    let userTotalCanvas = <HTMLCanvasElement>document.getElementById("usertotal");
    var userTotalCtx = userTotalCanvas.getContext("2d");
    new Chart(userTotalCtx, {
        type: "bar",
        data: usertotalData,
        options: {
            title: {
                display: true,
                text: getString("str-usertotal-title").replace("{}", progress.date),
            },
            scales: {
                xAxes: [{
                    ticks: { suggestedMin: 0, },
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-usertotal-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-usertotal-y-axis"),
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
    let progressCanvas = <HTMLCanvasElement>document.getElementById("progress");
    var progressCtx = progressCanvas.getContext("2d");
    new Chart(progressCtx, {
        type: "horizontalBar",
        data: progressData,
        options: {
            title: {
                display: true,
                text: getString("str-progress-title").replace("{1}", progress.percentage).replace("{2}", progress.date),
            },
            scales: {
                xAxes: [{
                    ticks: { min: 0.0, },
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-progress-x-axis"),
                    },
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: getString("str-progress-y-axis"),
                    },
                }]
            },
            plugins: {
                datalabels: {
                    // eslint-disable-next-line no-unused-vars
                    formatter: function(value: number, context: any) {
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
document.addEventListener("DOMContentLoaded", async function(event) {
    if (!document.getElementById("daily")) {
        // Not on the stats page.
        return;
    }

    var statsJSON = config.uriPrefix + "/static/stats.json";
    var response = await window.fetch(statsJSON);
    var stats = await response.json();
    addCharts(stats);
});

// vim: shiftwidth=4 softtabstop=4 expandtab:
