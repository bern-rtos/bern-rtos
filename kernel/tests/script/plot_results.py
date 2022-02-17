#!/usr/bin/pyhton

# Plot results previously stored in CSV file

import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import matplotlib


def main():
    sns.set(style="ticks")
    matplotlib.rcParams['font.sans-serif'] = "Fira Sans"
    matplotlib.rcParams['font.size'] = 10

    print("Read raw data")
    latencies = pd.read_csv("result/raw.csv", index_col=0)

    # Simplify column names
    columns = latencies.columns
    new_columns = []
    for column in columns:
        column = column.replace("arm_cm4-", "")
        column = column.replace("latency-", "")
        column = column.replace("_release", "")
        new_columns.append(column)
    latencies.columns = new_columns

    print("Plot histogram")
    ax = latencies.hist()
    for column in ax:
        for plot in column:
            plot.set_xlabel("latency / s")
    plt.savefig("result/hist.svg", bbox_inches='tight')

    print("Plot boxplot")
    plt.clf()
    latencies.boxplot()
    plt.ylabel("latency / s")
    plt.yscale('log')
    plt.xticks(rotation=60)
    plt.savefig("result/box.svg", bbox_inches='tight')


if __name__ == "__main__":
    main()
