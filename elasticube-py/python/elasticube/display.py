"""
Jupyter notebook display integration for ElastiCube.

Provides rich HTML representations for ElastiCube objects in Jupyter notebooks.
"""

from typing import Optional


def add_jupyter_repr(elasticube_class):
    """Add Jupyter notebook HTML representation to ElastiCube class.

    Args:
        elasticube_class: The PyElastiCube class to enhance
    """

    def _repr_html_(self):
        """Generate HTML representation for Jupyter notebooks."""
        name = self.name()
        row_count = self.row_count()

        html = f"""
        <div style="border: 2px solid #4CAF50; border-radius: 5px; padding: 15px;
                    background-color: #f9f9f9; font-family: Arial, sans-serif; max-width: 500px;">
            <h3 style="margin: 0 0 10px 0; color: #4CAF50;">
                <span style="font-size: 20px;">📊</span> ElastiCube
            </h3>
            <table style="width: 100%; border-collapse: collapse;">
                <tr>
                    <td style="padding: 5px; font-weight: bold; color: #555;">Name:</td>
                    <td style="padding: 5px; color: #333;">{name}</td>
                </tr>
                <tr style="background-color: #f0f0f0;">
                    <td style="padding: 5px; font-weight: bold; color: #555;">Rows:</td>
                    <td style="padding: 5px; color: #333;">{row_count:,}</td>
                </tr>
            </table>
            <div style="margin-top: 10px; padding: 8px; background-color: #e8f5e9;
                        border-left: 3px solid #4CAF50; font-size: 12px; color: #555;">
                💡 <strong>Tip:</strong> Use <code>.query()</code> to start querying this cube
            </div>
        </div>
        """
        return html

    def __repr__(self):
        """Generate string representation for terminal."""
        name = self.name()
        row_count = self.row_count()
        return f"ElastiCube(name='{name}', rows={row_count:,})"

    # Add methods to class
    elasticube_class._repr_html_ = _repr_html_
    elasticube_class.__repr__ = __repr__

    return elasticube_class


def add_querybuilder_repr(querybuilder_class):
    """Add Jupyter notebook HTML representation to QueryBuilder class.

    Args:
        querybuilder_class: The PyQueryBuilder class to enhance
    """

    def _repr_html_(self):
        """Generate HTML representation for Jupyter notebooks."""
        html = """
        <div style="border: 2px solid #2196F3; border-radius: 5px; padding: 15px;
                    background-color: #f9f9f9; font-family: Arial, sans-serif; max-width: 600px;">
            <h3 style="margin: 0 0 10px 0; color: #2196F3;">
                <span style="font-size: 20px;">🔍</span> QueryBuilder
            </h3>
            <div style="padding: 10px; background-color: #e3f2fd; border-radius: 3px; margin-bottom: 10px;">
                <p style="margin: 5px 0; color: #555;"><strong>Available Methods:</strong></p>
                <ul style="margin: 5px 0; padding-left: 20px; color: #333;">
                    <li><code>.select(columns)</code> - Select columns to retrieve</li>
                    <li><code>.filter(condition)</code> - Filter rows by condition</li>
                    <li><code>.group_by(columns)</code> - Group by columns</li>
                    <li><code>.order_by(columns)</code> - Sort results</li>
                    <li><code>.limit(n)</code> - Limit number of rows</li>
                </ul>
            </div>
            <div style="padding: 10px; background-color: #fff3e0; border-radius: 3px; margin-bottom: 10px;">
                <p style="margin: 5px 0; color: #555;"><strong>Execute Query:</strong></p>
                <ul style="margin: 5px 0; padding-left: 20px; color: #333;">
                    <li><code>.execute()</code> - Returns PyArrow Table</li>
                    <li><code>.to_pandas()</code> - Returns Pandas DataFrame</li>
                    <li><code>.to_polars()</code> - Returns Polars DataFrame (fastest)</li>
                </ul>
            </div>
            <div style="padding: 10px; background-color: #f3e5f5; border-radius: 3px;">
                <p style="margin: 5px 0; color: #555;"><strong>Visualization:</strong></p>
                <ul style="margin: 5px 0; padding-left: 20px; color: #333;">
                    <li><code>.plot().bar(x, y)</code> - Create bar chart</li>
                    <li><code>.plot().line(x, y)</code> - Create line chart</li>
                    <li><code>.plot().heatmap(x, y, values)</code> - Create heatmap</li>
                </ul>
            </div>
        </div>
        """
        return html

    def __repr__(self):
        """Generate string representation for terminal."""
        return "QueryBuilder(use .execute(), .to_pandas(), or .to_polars() to run query)"

    # Add methods to class
    querybuilder_class._repr_html_ = _repr_html_
    querybuilder_class.__repr__ = __repr__

    return querybuilder_class


def enable_jupyter_integration():
    """Enable Jupyter notebook integration for ElastiCube classes.

    This function should be called automatically when the module is imported
    in a Jupyter environment.
    """
    try:
        # Check if we're in a Jupyter environment
        from IPython import get_ipython
        ipython = get_ipython()

        if ipython is not None:
            # We're in IPython/Jupyter, enable rich display
            return True
    except ImportError:
        pass

    return False
