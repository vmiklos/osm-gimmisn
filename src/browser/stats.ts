/*
 * Copyright 2020 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

import {
    Chart,
    LineElement,
    BarElement,
    PointElement,
    BarController,
    LineController,
    CategoryScale,
    LinearScale,
    Legend,
    Filler,
    Title
} from 'chart.js';
import ChartDataLabels from 'chartjs-plugin-datalabels';
import * as ChartDatalabels from "chartjs-plugin-datalabels/types/context";
import ChartTrendline from "chartjs-plugin-trendline";
import * as config from './config';

Chart.register(
    LineElement,
    BarElement,
    PointElement,
    BarController,
    LineController,
    CategoryScale,
    LinearScale,
    Legend,
    Filler,
    Title,
    ChartTrendline,
    ChartDataLabels
);

function getString(key: string): string {
    const element = document.getElementById(key);
    if (!element) {
        return '';
    }
    const value = element.getAttribute("data-value");
    if (!value) {
        return '';
    }

    return value;
}

// StatsProgress is the "progress" / "capital-progress" key of workdir/stats/stats.json.
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
    'capital-progress': StatsProgress;
    invalidAddrCities: Array<[string, number]>;
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
    const capitalProgress = stats['capital-progress'];
    const trendlineOptions = {
        style: "rgba(255,105,180, .8)",
        lineStyle: "dotted",
        width: 2,
    };
    const invalidAddrCities = stats.invalidAddrCities;

    const dailyData = {
        // daily is a list of label-data pairs.
        labels: daily.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: daily.map(function(x: [string, number]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    const dailyCanvas = document.getElementById("daily") as HTMLCanvasElement;
    const dailyCtx = dailyCanvas.getContext("2d");
    if (!dailyCtx) {
        return;
    }
    new Chart(dailyCtx, {
        type: "bar",
        data: dailyData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-daily-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    suggestedMin: 0,
                    title: {
                        display: true,
                        text: getString("str-daily-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-daily-y-axis"),
                    },
                }
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
    const monthlyCanvas = document.getElementById("monthly") as HTMLCanvasElement;
    const monthlyCtx = monthlyCanvas.getContext("2d");
    if (!monthlyCtx) {
        return;
    }
    new Chart(monthlyCtx, {
        type: "bar",
        data: monthlyData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-monthly-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    suggestedMin: 0,
                    title: {
                        display: true,
                        text: getString("str-monthly-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-monthly-y-axis"),
                    },
                }
            },
        }
    });

    const monthlytotalData = {
        // monthlytotal is a list of label-data pairs.
        labels: monthlytotal.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: monthlytotal.map(function(x: [string, number]) { return x[1]; }),
            fill: true,
            trendlineLinear: trendlineOptions,
        }]
    };
    const monthlyTotalCanvas = document.getElementById("monthlytotal") as HTMLCanvasElement;
    const monthlyTotalCtx = monthlyTotalCanvas.getContext("2d");
    if (!monthlyTotalCtx) {
        return;
    }
    new Chart(monthlyTotalCtx, {
        type: "line",
        data: monthlytotalData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-monthlytotal-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    title: {
                        display: true,
                        text: getString("str-monthlytotal-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-monthlytotal-y-axis"),
                    },
                }
            },
        }
    });

    const dailytotalData = {
        // dailytotal is a list of label-data pairs.
        labels: dailytotal.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: dailytotal.map(function(x: [string, number]) { return x[1]; }),
            fill: true,
            trendlineLinear: trendlineOptions,
        }]
    };

    const dailyTotalCanvas = document.getElementById("dailytotal") as HTMLCanvasElement;
    const dailyTotalCtx = dailyTotalCanvas.getContext("2d");
    if (!dailyTotalCtx) {
        return;
    }
    new Chart(dailyTotalCtx, {
        type: "line",
        data: dailytotalData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-dailytotal-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    title: {
                        display: true,
                        text: getString("str-dailytotal-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-dailytotal-y-axis"),
                    },
                }
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
    const topUsersCanvas = document.getElementById("topusers") as HTMLCanvasElement;
    const topUsersCtx = topUsersCanvas.getContext("2d");
    if (!topUsersCtx) {
        return;
    }
    new Chart(topUsersCtx, {
        type: "bar",
        data: topusersData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-topusers-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    suggestedMin: 0,
                    title: {
                        display: true,
                        text: getString("str-topusers-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-topusers-y-axis"),
                    },
                }
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
    const topCitiesCanvas = document.getElementById("topcities") as HTMLCanvasElement;
    const topCitiesCtx = topCitiesCanvas.getContext("2d");
    if (!topCitiesCtx) {
        return;
    }
    new Chart(topCitiesCtx, {
        type: "bar",
        data: topcitiesData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-topcities-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    suggestedMin: 0,
                    title: {
                        display: true,
                        text: getString("str-topcities-x-axis"),
                    },
                },
                y: {
                    beginAtZero: false, // default would be false
                    title: {
                        display: true,
                        text: getString("str-topcities-y-axis"),
                    },
                }
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
    const userTotalCanvas = document.getElementById("usertotal") as HTMLCanvasElement;
    const userTotalCtx = userTotalCanvas.getContext("2d");
    if (!userTotalCtx) {
        return;
    }
    new Chart(userTotalCtx, {
        type: "bar",
        data: usertotalData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-usertotal-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    suggestedMin: 0,
                    title: {
                        display: true,
                        text: getString("str-usertotal-x-axis"),
                    },
                },
                y: {
                    beginAtZero: false, // default would be false
                    title: {
                        display: true,
                        text: getString("str-usertotal-y-axis"),
                    },
                }
            },
        }
    });

    const progressData = {
        // One data set has a single value here, so no visible label is needed.
        labels: [""],
        datasets: [{
            label: getString("str-reference"),
            backgroundColor: "rgba(255, 0, 0, 0.5)",
            data: [ progress.reference ],
        }, {
            label: "OSM",
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: [ progress.osm ],
        }]

    };
    const progressCanvas = document.getElementById("progress") as HTMLCanvasElement;
    const progressCtx = progressCanvas.getContext("2d");
    if (!progressCtx) {
        return;
    }
    new Chart(progressCtx, {
        type: "bar",
        data: progressData,
        options: {
            indexAxis: "y",
            plugins: {
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-progress-title").replace("{1}", progress.percentage.toString()).replace("{2}", progress.date),
                },
                datalabels: {
                    // eslint-disable-next-line @typescript-eslint/no-unused-vars
                    formatter: function(value: number, context: ChartDatalabels.Context) {
                        // Turn 1000 into '1 000'.
                        return value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, " ");
                    }
                }
            },
            scales: {
                x: {
                    min: 0.0,
                    title: {
                        display: true,
                        text: getString("str-progress-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-progress-y-axis"),
                    },
                }
            },
        }
    });

    const capitalProgressData = {
        labels: [""],
        datasets: [{
            label: getString("str-reference"),
            backgroundColor: "rgba(255, 0, 0, 0.5)",
            data: [ capitalProgress.reference ],
        }, {
            label: "OSM",
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: [ capitalProgress.osm ],
        }]

    };
    const capitalProgressCanvas = document.getElementById("capital-progress") as HTMLCanvasElement;
    const capitalProgressCtx = capitalProgressCanvas.getContext("2d");
    if (!capitalProgressCtx) {
        return;
    }
    new Chart(capitalProgressCtx, {
        type: "bar",
        data: capitalProgressData,
        options: {
            indexAxis: "y",
            plugins: {
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-capital-progress-title").replace("{1}", capitalProgress.percentage.toString()).replace("{2}", capitalProgress.date),
                },
                datalabels: {
                    // eslint-disable-next-line @typescript-eslint/no-unused-vars
                    formatter: function(value: number, context: ChartDatalabels.Context) {
                        // Turn 1000 into '1 000'.
                        return value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, " ");
                    }
                }
            },
            scales: {
                x: {
                    min: 0.0,
                    title: {
                        display: true,
                        text: getString("str-capital-progress-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-progress-y-axis"),
                    },
                }
            },
        }
    });

    const invalidAddrCitiesData = {
        // invalidAddrCities is a list of label-data pairs.
        labels: invalidAddrCities.map(function(x: [string, number]) { return x[0]; }),
        datasets: [{
            backgroundColor: "rgba(0, 255, 0, 0.5)",
            data: invalidAddrCities.map(function(x: [string, number]) { return x[1]; }),
            trendlineLinear: trendlineOptions,
        }]
    };
    const invalidAddrCitiesCanvas = document.getElementById("stats-invalid-addr-cities") as HTMLCanvasElement;
    const invalidAddrCitiesCtx = invalidAddrCitiesCanvas.getContext("2d");
    if (!invalidAddrCitiesCtx) {
        return;
    }
    new Chart(invalidAddrCitiesCtx, {
        type: "bar",
        data: invalidAddrCitiesData,
        options: {
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: true,
                    padding: 30, // default would be 10, which may overlap
                    text: getString("str-invalid-addr-cities-title").replace("{}", progress.date),
                },
                datalabels: {
                    align: "top",
                    anchor: "end",
                }
            },
            scales: {
                x: {
                    suggestedMin: 0,
                    title: {
                        display: true,
                        text: getString("str-invalid-addr-cities-x-axis"),
                    },
                },
                y: {
                    title: {
                        display: true,
                        text: getString("str-invalid-addr-cities-y-axis"),
                    },
                }
            },
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
    const stats = await response.json();
    addCharts(stats);
    return;
}

export { initStats, getString };

// vim: shiftwidth=4 softtabstop=4 expandtab:
