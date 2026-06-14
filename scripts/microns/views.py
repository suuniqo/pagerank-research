import pandas as pd
import numpy
import conntility
import plotly.graph_objects as go
import plotly.express as px

fn_mat = "../../data/microns/microns_mm3_connectome_v1181.h5"
name_dset_f = "full"
name_dset_c = "condensed"
M = conntility.ConnectivityMatrix.from_h5(fn_mat, name_dset_f)
C = conntility.ConnectivityMatrix.from_h5(fn_mat, name_dset_c)

comm_df = pd.read_csv("../../out/microns/communities.csv", header=None, names=["index", "community"])
community = (
    comm_df
    .set_index("index")
    .loc[C.vertices["index"], "community"]
    .values
)
C.add_vertex_property("community", community)

palette = px.colors.qualitative.Bold

def make_2d_fig(x_col, y_col, x_title, y_title):
    sel = numpy.random.choice(len(C.vertices), 70000, replace=False)  # same as 3d
    df = C.vertices.iloc[sel]
    fig = go.Figure()
    for i, (name, group) in enumerate(df.groupby("community")):
        fig.add_trace(go.Scattergl(          # ← WebGL, much faster
            x=group[x_col],
            y=group[y_col],
            mode='markers',
            name=f"Community {name}",
            marker=dict(
                size=4,                      # bigger, matching 3d feel
                color=palette[i % len(palette)],
                opacity=1,
                line=dict(width=0),
            )
        ))
    fig.update_layout(
        paper_bgcolor='#ffffff',
        plot_bgcolor='#f5f5f8',
        showlegend=True,
        legend=dict(
            title=dict(text='Community', font=dict(size=13, color='#333333')),
            itemsizing='constant',
            bgcolor='rgba(255,255,255,0.85)',
            bordercolor='rgba(0,0,0,0.08)',
            borderwidth=1,
            font=dict(size=11, color='#333333'),
            x=1.01,
            xanchor='left',
            y=0.5,
            yanchor='middle',
        ),
        xaxis=dict(
            title=x_title,
            gridcolor='#ccccdd',
            showgrid=False,
            zeroline=False,
            color='#333333',
            scaleanchor='y',
            scaleratio=1,
        ),
        yaxis=dict(
            title=y_title,
            gridcolor='#ccccdd',
            showgrid=False,
            zeroline=False,
            color='#333333',
        ),
        margin=dict(l=0, r=160, b=0, t=0),
        font=dict(color='#333333'),
    )
    fig.show()

# surface graph
make_2d_fig("x_nm", "z_nm", "X (nm)", "Z (nm)")

# depth graph
make_2d_fig("x_nm", "y_nm", "X (nm)", "Y (nm)")

# 3d graph
sel = numpy.random.choice(len(C.vertices), 70000, replace=False)
df_plot = C.vertices.iloc[sel]
fig = go.Figure()
for i, (name, group) in enumerate(df_plot.groupby("community")):
    fig.add_trace(go.Scatter3d(
        x=group["x_nm"],
        y=group["y_nm"],
        z=group["z_nm"],
        mode='markers',
        name=f"Community {name}",
        marker=dict(
            size=2.5,
            color=palette[i % len(palette)],
            opacity=1,
            line=dict(width=0),
        )
    ))
fig.update_layout(
    paper_bgcolor='#ffffff',
    showlegend=False,
    scene=dict(
        bgcolor='#f5f5f8',
        xaxis=dict(
            title='X (nm)',
            gridcolor='#ccccdd',
            showbackground=False,
            color='#333333',
        ),
        yaxis=dict(
            title='Y (nm)',
            gridcolor='#ccccdd',
            showbackground=False,
            color='#333333',
        ),
        zaxis=dict(
            title='Z (nm)',
            gridcolor='#ccccdd',
            showbackground=False,
            color='#333333',
        ),
    ),
    margin=dict(l=0, r=0, b=0, t=0),
    font=dict(color='#333333'),
)
fig.show()
