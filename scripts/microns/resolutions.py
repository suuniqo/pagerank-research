import pandas as pd
import numpy
import conntility
from matplotlib import pyplot as plt

fn_mat = "../../data/microns/microns_mm3_connectome_v1181.h5"
name_dset_f = "full"
name_dset_c = "condensed"

M = conntility.ConnectivityMatrix.from_h5(fn_mat, name_dset_f)
C = conntility.ConnectivityMatrix.from_h5(fn_mat, name_dset_c)

for res in numpy.arange(1.0, 5.5, 0.5):
    print(f"{res=}")
    comm_df = pd.read_csv(f"../../out/microns/res/communities-{res:g}.csv", header=None, names=["index", "community"])

    community = (
        comm_df
        .set_index("index")
        .loc[C.vertices["index"], "community"]
        .values
    )
    C.add_vertex_property("community", community, overwrite=True)

    # superficial graph

    sel = numpy.random.choice(len(C.vertices), 24000, replace=False)

    df = C.vertices.iloc[sel]

    fig, ax = plt.subplots(figsize=(14, 7))

    handles = []
    for name, group in df.groupby("community"):
        line, = ax.plot(group["x_nm"], group["z_nm"], ls="None", marker=".", label=f"Community {name}")
        handles.append(line)

    ax.axis("equal")
    ax.legend(handles=handles, title="Community", bbox_to_anchor=(1.05, 1), loc="upper left", borderaxespad=0)
    plt.tight_layout()
    plt.title(f"Surface. Resolution = {res}")
    plt.show()

    # depth graph

    sel = numpy.random.choice(len(C.vertices), 24000, replace=False)

    df = C.vertices.iloc[sel]

    fig, ax = plt.subplots(figsize=(14, 7))

    handles = []
    for name, group in df.groupby("community"):
        line, = ax.plot(group["x_nm"], group["y_nm"], ls="None", marker=".", label=f"Community {name}")
        handles.append(line)

    ax.axis("equal")
    ax.legend(handles=handles, title="Community", bbox_to_anchor=(1.05, 1), loc="upper left", borderaxespad=0)
    plt.tight_layout()
    plt.title(f"Depth. Resolution = {res}")
    plt.show()

    # 3d graph

    # sel = numpy.random.choice(len(C.vertices), 10000, replace=False)

    # df = C.vertices.iloc[sel]

    # fig = plt.figure(figsize=(14, 9))
    # ax = fig.add_subplot(111, projection='3d')

    # handles = []
    # for name, group in df.groupby("community"):
    #     sc = ax.scatter(group["x_nm"], group["y_nm"], group["z_nm"],
    #                     s=1, label=f"Community {name}")
    #     handles.append(sc)

    # ax.set_xlabel("X")
    # ax.set_ylabel("Y")
    # ax.set_zlabel("Z")
    # ax.legend(handles=handles, title="Community", bbox_to_anchor=(1.05, 1), loc="upper left", borderaxespad=0)
    # plt.tight_layout()
    # plt.show()
