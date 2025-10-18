"""
Visualization helpers for ElastiCube.

Provides integration with matplotlib and seaborn for visualizing cube data.
"""

from typing import Optional, List, Union, Dict, Any


class CubeVisualizer:
    """Helper class for visualizing ElastiCube query results."""

    def __init__(self, query_builder):
        """Initialize with a query builder.

        Args:
            query_builder: A QueryBuilder instance
        """
        self.query_builder = query_builder

    def bar(
        self,
        x: str,
        y: str,
        title: Optional[str] = None,
        xlabel: Optional[str] = None,
        ylabel: Optional[str] = None,
        figsize: tuple = (10, 6),
        color: Optional[str] = None,
        **kwargs
    ):
        """Create a bar chart from query results.

        Args:
            x: Column name for x-axis
            y: Column name for y-axis (typically an aggregated measure)
            title: Chart title
            xlabel: X-axis label
            ylabel: Y-axis label
            figsize: Figure size as (width, height)
            color: Bar color
            **kwargs: Additional arguments passed to matplotlib bar()

        Returns:
            matplotlib Figure and Axes objects

        Raises:
            ImportError: If matplotlib is not installed
        """
        try:
            import matplotlib.pyplot as plt
        except ImportError:
            raise ImportError(
                "matplotlib is required for visualization. "
                "Install with: pip install matplotlib"
            )

        # Get data
        df = self.query_builder.to_pandas()

        # Create plot
        fig, ax = plt.subplots(figsize=figsize)
        ax.bar(df[x], df[y], color=color, **kwargs)

        # Set labels
        ax.set_xlabel(xlabel or x)
        ax.set_ylabel(ylabel or y)
        if title:
            ax.set_title(title)

        plt.xticks(rotation=45, ha='right')
        plt.tight_layout()

        return fig, ax

    def line(
        self,
        x: str,
        y: Union[str, List[str]],
        title: Optional[str] = None,
        xlabel: Optional[str] = None,
        ylabel: Optional[str] = None,
        figsize: tuple = (10, 6),
        **kwargs
    ):
        """Create a line chart from query results.

        Args:
            x: Column name for x-axis
            y: Column name(s) for y-axis (can be a list for multiple lines)
            title: Chart title
            xlabel: X-axis label
            ylabel: Y-axis label
            figsize: Figure size as (width, height)
            **kwargs: Additional arguments passed to matplotlib plot()

        Returns:
            matplotlib Figure and Axes objects

        Raises:
            ImportError: If matplotlib is not installed
        """
        try:
            import matplotlib.pyplot as plt
        except ImportError:
            raise ImportError(
                "matplotlib is required for visualization. "
                "Install with: pip install matplotlib"
            )

        # Get data
        df = self.query_builder.to_pandas()

        # Create plot
        fig, ax = plt.subplots(figsize=figsize)

        if isinstance(y, list):
            for col in y:
                ax.plot(df[x], df[col], label=col, **kwargs)
            ax.legend()
        else:
            ax.plot(df[x], df[y], **kwargs)

        # Set labels
        ax.set_xlabel(xlabel or x)
        ax.set_ylabel(ylabel or (y if isinstance(y, str) else 'Value'))
        if title:
            ax.set_title(title)

        plt.xticks(rotation=45, ha='right')
        plt.tight_layout()

        return fig, ax

    def heatmap(
        self,
        x: str,
        y: str,
        values: str,
        title: Optional[str] = None,
        figsize: tuple = (12, 8),
        cmap: str = 'YlOrRd',
        **kwargs
    ):
        """Create a heatmap from query results.

        Args:
            x: Column name for x-axis
            y: Column name for y-axis
            values: Column name for heatmap values
            title: Chart title
            figsize: Figure size as (width, height)
            cmap: Colormap name
            **kwargs: Additional arguments passed to seaborn heatmap()

        Returns:
            matplotlib Figure and Axes objects

        Raises:
            ImportError: If seaborn is not installed
        """
        try:
            import matplotlib.pyplot as plt
            import seaborn as sns
        except ImportError:
            raise ImportError(
                "seaborn and matplotlib are required for heatmaps. "
                "Install with: pip install matplotlib seaborn"
            )

        # Get data
        df = self.query_builder.to_pandas()

        # Pivot for heatmap
        pivot_df = df.pivot(index=y, columns=x, values=values)

        # Create plot
        fig, ax = plt.subplots(figsize=figsize)
        sns.heatmap(pivot_df, annot=True, fmt='.0f', cmap=cmap, ax=ax, **kwargs)

        if title:
            ax.set_title(title)

        plt.tight_layout()

        return fig, ax

    def scatter(
        self,
        x: str,
        y: str,
        size: Optional[str] = None,
        color: Optional[str] = None,
        title: Optional[str] = None,
        xlabel: Optional[str] = None,
        ylabel: Optional[str] = None,
        figsize: tuple = (10, 6),
        **kwargs
    ):
        """Create a scatter plot from query results.

        Args:
            x: Column name for x-axis
            y: Column name for y-axis
            size: Column name for point sizes (optional)
            color: Column name for point colors (optional)
            title: Chart title
            xlabel: X-axis label
            ylabel: Y-axis label
            figsize: Figure size as (width, height)
            **kwargs: Additional arguments passed to matplotlib scatter()

        Returns:
            matplotlib Figure and Axes objects

        Raises:
            ImportError: If matplotlib is not installed
        """
        try:
            import matplotlib.pyplot as plt
        except ImportError:
            raise ImportError(
                "matplotlib is required for visualization. "
                "Install with: pip install matplotlib"
            )

        # Get data
        df = self.query_builder.to_pandas()

        # Create plot
        fig, ax = plt.subplots(figsize=figsize)

        scatter_kwargs = kwargs.copy()
        if size:
            scatter_kwargs['s'] = df[size]
        if color:
            scatter_kwargs['c'] = df[color]

        ax.scatter(df[x], df[y], **scatter_kwargs)

        # Set labels
        ax.set_xlabel(xlabel or x)
        ax.set_ylabel(ylabel or y)
        if title:
            ax.set_title(title)

        plt.tight_layout()

        return fig, ax

    def pie(
        self,
        labels: str,
        values: str,
        title: Optional[str] = None,
        figsize: tuple = (8, 8),
        **kwargs
    ):
        """Create a pie chart from query results.

        Args:
            labels: Column name for pie slice labels
            values: Column name for pie slice values
            title: Chart title
            figsize: Figure size as (width, height)
            **kwargs: Additional arguments passed to matplotlib pie()

        Returns:
            matplotlib Figure and Axes objects

        Raises:
            ImportError: If matplotlib is not installed
        """
        try:
            import matplotlib.pyplot as plt
        except ImportError:
            raise ImportError(
                "matplotlib is required for visualization. "
                "Install with: pip install matplotlib"
            )

        # Get data
        df = self.query_builder.to_pandas()

        # Create plot
        fig, ax = plt.subplots(figsize=figsize)
        ax.pie(df[values], labels=df[labels], autopct='%1.1f%%', **kwargs)

        if title:
            ax.set_title(title)

        plt.tight_layout()

        return fig, ax


def add_viz_methods(query_builder_class):
    """Add visualization methods to QueryBuilder class.

    This is a helper function to extend the QueryBuilder with viz methods.
    """

    def plot(self):
        """Get a visualizer for this query builder.

        Returns:
            CubeVisualizer instance for creating plots

        Example:
            >>> query = cube.query()
            >>> query.select(["region", "SUM(sales) as total"])
            >>> query.group_by(["region"])
            >>> fig, ax = query.plot().bar("region", "total", title="Sales by Region")
        """
        return CubeVisualizer(self)

    # Add method to class
    query_builder_class.plot = plot

    return query_builder_class
