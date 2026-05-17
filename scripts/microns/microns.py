import pandas as pd
import numpy
import conntility

fn_mat = "../../data/microns/microns_mm3_connectome_v1181.h5"
name_dset_f = "full"
name_dset_c = "condensed"

M = conntility.ConnectivityMatrix.from_h5(fn_mat, name_dset_f)
C = conntility.ConnectivityMatrix.from_h5(fn_mat, name_dset_c)

lines = {
    ("visp", "visrl"): [[948.9304812834225, 1168.3783783783783],
[948.9304812834225, 1118.6486486486485],
[979.9465240641712, 1050.2702702702702],
[1010.9625668449198, 1019.1891891891892],
[1041.9786096256685, 963.2432432432432],
[1072.9946524064171, 904.1891891891892],
[1107.1122994652405, 832.7027027027027],
[1144.331550802139, 755],
[1147.433155080214, 683.5135135135135],
[1166.042780748663, 633.7837837837837],
[1162.9411764705883, 615.1351351351351],
[1172.2459893048128, 602.7027027027027]
],
    ("visp", "vislm"): [[1175.3475935828876, 602.7027027027027],
[1162.9411764705883, 562.2972972972973],
[1172.2459893048128, 509.45945945945937],
[1184.6524064171124, 450.4054054054053],
[1209.4652406417113, 375.81081081081084],
[1234.2780748663101, 288.78378378378375],
[1243.5828877005347, 189.32432432432438],
[1252.8877005347595, 24.594594594594582],
[1255.9893048128342, -87.29729729729729]
],
    ("vislm", "visrl"): [[1175.3475935828876, 602.7027027027027],
[1215.668449197861, 602.7027027027027],
[1259.0909090909092, 627.5675675675675],
[1283.9037433155079, 627.5675675675675]
],
    ("visal", "visrl"): [[1283.9037433155079, 627.5675675675675],
[1302.5133689839572, 671.081081081081],
[1324.2245989304813, 727.0270270270271],
[1355.24064171123, 807.8378378378378],
[1411.0695187165777, 904.1891891891892],
[1470, 988.1081081081081],
[1559.9465240641712, 1068.918918918919],
[1569.2513368983957, 1068.918918918919]
],
    ("vislm", "visal"):[[1283.9037433155079, 630.6756756756756],
[1314.9197860962568, 593.3783783783783],
[1358.342245989305, 568.5135135135135],
[1414.1711229946525, 528.1081081081081],
[1470, 487.7027027027027],
[1532.0320855614975, 453.51351351351354],
[1572.3529411764705, 453.51351351351354]
]
}
lines = dict([(k, numpy.array(v)) for k, v in lines.items()])

# Plotting neuron densities and the digitized region borders.

from matplotlib import pyplot as plt

nbins= 51
extents = {}
for col in ["x_nm", "y_nm", "z_nm"]:
    bins = numpy.linspace(C.vertices[col].min(), C.vertices[col].max() + 1, nbins)
    extents[col] = [bins[0] / 1000, bins[-1] / 1000]
    C.add_vertex_property(col + "_binned_{0}".format(nbins),
                         numpy.digitize(C.vertices[col], bins=bins))
    
I = C.vertices.groupby(["x_nm_binned_51",
                        "z_nm_binned_51"])["index"].count().unstack("x_nm_binned_51")

plt.colorbar(plt.imshow(I.values,
            extent=extents["x_nm"] + extents["z_nm"][::-1]))
plt.contour(I.values, cmap="hot", levels=[30.],
           extent=extents["x_nm"] + extents["z_nm"])
plt.gca().set_ylim(sorted(plt.gca().get_ylim()))

for k, v in lines.items():
    v
    plt.plot(v[:, 0], v[:, 1], color="red")
    
pt = numpy.array([1000, 800])
plt.plot(*pt, 'ro')

plt.show()

# # Algorithm to resolve which region a point is in, based on the region borders.
#
# def distance_and_angle(lineseg, pt):
#     dl = numpy.diff(lineseg, axis=0)[0]
#     dl_n = dl / numpy.linalg.norm(dl)
#     nrml = numpy.array([-dl_n[1], dl_n[0]])
#
#     dp = pt - lineseg[0]
#     side = numpy.sign(numpy.dot(nrml, dp))
#
#     return side, numpy.linalg.norm(lineseg.mean(axis=0) - pt)
#
# def resolve_side(ln, pt):
#     side, dist = [list(_x) for _x in
#         zip(*[
#         distance_and_angle(ln[i:(i+2)], pt)
#         for i in range(len(ln) - 1)
#     ])]
#     return side[numpy.argmin(dist)]
#
# def resolve(pt):
#     lst_regions = ["visp", "visrl", "vislm", "visal"]
#     ruled_out = []
#     for border, ln in lines.items():
#         res = resolve_side(ln, pt)
#         ruled_out.append(border[int(res < 0)])
#     res_region = numpy.setdiff1d(lst_regions, ruled_out)
#     assert len(res_region) == 1
#     return lst_regions.index(res_region[0])
#
# resolve(pt) # 0: VISP
#
# # Test for a grid of points
#
# X, Y = numpy.meshgrid(numpy.linspace(300, 1600, 50),
#                      numpy.linspace(600, 1100, 50))
# pts = numpy.vstack([X.flatten(), Y.flatten()]).transpose()
#
# import tqdm
# lst_regions = ["visp", "visrl", "vislm", "visal"]
#
# reg = [resolve(pt) for pt in tqdm.tqdm(pts)]
# reg = numpy.array(reg).reshape(X.shape)
#
# plt.imshow(reg)
# plt.show()
#
# # For all neurons
#
# tentative_region = C.vertices[["x_nm", "z_nm"]].apply(lambda x: resolve(x.values / 1000), axis=1)
#
# C.add_vertex_property("tentative_region", numpy.array(lst_regions)[tentative_region])
#
# print(C.vertices)
#
# # Plot some examples
#
# sel = numpy.random.choice(len(C.vertices), 4000, replace=False)
# df = C.vertices.iloc[sel]
#
# df.groupby("tentative_region").apply(
#     lambda x: plt.plot(x["x_nm"], x["z_nm"], ls="None", marker='.')
# )
# plt.axis("equal")
#
# plt.show()

# Plot the communitites

comm_df = pd.read_csv("../../out/microns/communities.csv", header=None, names=["index", "community"])

community = (
    comm_df
    .set_index("index")
    .loc[C.vertices["index"], "community"]
    .values
)

C.add_vertex_property("community", community)


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
plt.show()

# 3d graph

sel = numpy.random.choice(len(C.vertices), 10000, replace=False)

df = C.vertices.iloc[sel]

fig = plt.figure(figsize=(14, 9))
ax = fig.add_subplot(111, projection='3d')

handles = []
for name, group in df.groupby("community"):
    sc = ax.scatter(group["x_nm"], group["y_nm"], group["z_nm"],
                    s=1, label=f"Community {name}")
    handles.append(sc)

ax.set_xlabel("X")
ax.set_ylabel("Y")
ax.set_zlabel("Z")
ax.legend(handles=handles, title="Community", bbox_to_anchor=(1.05, 1), loc="upper left", borderaxespad=0)
plt.tight_layout()
plt.show()
