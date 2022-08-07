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
    matplotlib.rcParams['axes.titlesize'] = 14
    matplotlib.rcParams['axes.titleweight'] = "bold"

    print("Read raw data")
    latencies = pd.read_csv("result/arm_cm4-combined.csv", index_col=0)

    stats = latencies.describe()
    stats.to_csv("result/stats.csv")


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
    for column in latencies.columns:
        plt.clf()
        sns.histplot(latencies, x=column, stat="probability")
        #sns.swarmplot(data=latencies, x=column)
        plt.title(column)
        plt.xlabel("Latency / s")
        plt.savefig("result/{}-hist.svg".format(column), bbox_inches='tight')

    plt.clf()
    sns.histplot(latencies, bins=100)
    plt.title("Latency combined")
    plt.xlabel("Latency / s")
    plt.savefig("result/hist.svg", bbox_inches='tight')

    print("Plot boxplot")
    plt.clf()
    sns.boxplot(data=latencies)
    plt.ylabel("Latency / s")
    plt.yscale('log')
    plt.xticks(rotation=60)
    plt.savefig("result/box.svg", bbox_inches='tight')


if __name__ == "__main__":
    main()
