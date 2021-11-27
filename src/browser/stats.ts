/*
 * Copyright 2020 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

import Chart = require("chart.js");
import "chartjs-plugin-datalabels"; // only for its side-effects
import * as ChartDatalabels from "chartjs-plugin-datalabels/types/context";
import chartJsTrendline = require("chartjs-plugin-trendline");
Chart.plugins.register(chartJsTrendline);

import * as config from './config';

function getString(key: string) {
    return document.getElementById(key).getAttribute("data-value");
}

// StatsProgress is the "progress" key of workdir/stats/stats.json.
interface StatsProgress {
    date: string;
    percentage: number;
    reference: number;
    osm: number;
}

// Stats is the root of workdir/stats/stats.json.
interface Stats {
    daily: Array<[string, number]>;
    dailytotal: Array<[string, number]>;
    monthly: Array<[string, number]>;
    monthlytotal: Array<[string, number]>;
    topusers: Array<[string, number]>;
    topcities: Array<[string, number]>;
    usertotal: Array<[string, number]>;
    progress: StatsProgress;
}

function addCharts(stats: Stats) {
    const daily = stats.daily;
    const dailytotal = stats.dailytotal;
    const monthly = stats.monthly;
    const monthlytotal = stats.monthlytotal;
    const topusers = stats.topusers;
    const topcities = stats.topcities;
    const usertotal = stats.usertotal;
    const progress = stats.progress;
    const trendlineOptions = {
        style: "rgba(255,105,180, .8)",
        lineStyle: "dotted",
        width: 2,
    };

    const dailyData = {
        // daily is a list of label-data pairs.
        labels: daily.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: daily.map(function(x: [string, number]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    const dailyCanvas = <HTMLCanvasElement>document.getElementById("daily");
    const dailyCtx = dailyCanvas.getContext("2d");
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

    const monthlyData = {
        // monthly is a list of label-data pairs.
        labels: monthly.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthly.map(function(x: [string, number]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    const monthlyCanvas = <HTMLCanvasElement>document.getElementById("monthly");
    const monthlyCtx = monthlyCanvas.getContext("2d");
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

    const monthlytotalData = {
        // monthlytotal is a list of label-data pairs.
        labels: monthlytotal.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthlytotal.map(function(x: [string, number]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    const monthlyTotalCanvas = <HTMLCanvasElement>document.getElementById("monthlytotal");
    const monthlyTotalCtx = monthlyTotalCanvas.getContext("2d");
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

    const dailytotalData = {
        // dailytotal is a list of label-data pairs.
        labels: dailytotal.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: dailytotal.map(function(x: [string, number]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };

    const dailyTotalCanvas = <HTMLCanvasElement>document.getElementById("dailytotal");
    const dailyTotalCtx = dailyTotalCanvas.getContext("2d");
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

    const topusersData = {
        // topusers is a list of label-data pairs.
        labels: topusers.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: topusers.map(function(x: [string, number]) { return x[1]; }),
        }]

    };
    const topUsersCanvas = <HTMLCanvasElement>document.getElementById("topusers");
    const topUsersCtx = topUsersCanvas.getContext("2d");
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
    const topcitiesData = {
        // topcities is a list of label-data pairs.
        labels: topcities.map(function(x: [string, number]) {
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
            data: topcities.map(function(x: [string, number]) { return x[1]; }),
        }]

    };
    const topCitiesCanvas = <HTMLCanvasElement>document.getElementById("topcities");
    const topCitiesCtx = topCitiesCanvas.getContext("2d");
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

    const usertotalData = {
        // usertotal is a list of label-data pairs.
        labels: usertotal.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: usertotal.map(function(x: [string, number]) { return x[1]; }),
        }]

    };
    const userTotalCanvas = <HTMLCanvasElement>document.getElementById("usertotal");
    const userTotalCtx = userTotalCanvas.getContext("2d");
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

    const progressData = {
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
    const progressCanvas = <HTMLCanvasElement>document.getElementById("progress");
    const progressCtx = progressCanvas.getContext("2d");
    new Chart(progressCtx, {
        type: "horizontalBar",
        data: progressData,
        options: {
            title: {
                display: true,
                text: getString("str-progress-title").replace("{1}", progress.percentage.toString()).replace("{2}", progress.date),
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
                    // eslint-disable-next-line @typescript-eslint/no-unused-vars
                    formatter: function(value: number, context: ChartDatalabels.Context) {
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

async function initStats(): Promise<void>
{
    if (!document.getElementById("daily")) {
        // Not on the stats page.
        return;
    }

    const statsJSON = config.uriPrefix + "/static/stats.json";
    const response = await window.fetch(statsJSON);
    const stats = await<Promise<Stats>> response.json();
    addCharts(stats);
    return;
}

export { initStats };

// vim: shiftwidth=4 softtabstop=4 expandtab:
