#!/usr/bin/env python3
"""mtop - Money-based resource monitor TUI."""
from textual.app import App, ComposeResult
from textual.containers import Container, Horizontal, Vertical, ScrollableContainer
from textual.widgets import Header, Footer, Static, DataTable
from textual.reactive import reactive
from rich.text import Text
from datetime import datetime

from hardware import HardwareInfo


class CPUVisualizer(Static):
    """ASCII art CPU visualization with dot matrix."""

    def __init__(self, hw: HardwareInfo, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.hw = hw

    def render(self) -> Text:
        """Render CPU visualization with dots."""
        cpu_data = self.hw.get_cpu_usage_cost()
        per_core = cpu_data['per_core']

        content = Text()

        # Create dot matrix representation of CPU activity
        # Grid of dots representing overall CPU
        rows = 12
        cols = 20

        # Overall CPU usage as dot density
        density = int((cpu_data['percent'] / 100.0) * rows * cols)

        content.append("      ┌", style="dim")
        content.append("─" * (cols + 2), style="dim")
        content.append("┐\n", style="dim")

        for row in range(rows):
            content.append("      │ ", style="dim")
            for col in range(cols):
                idx = row * cols + col
                if idx < density:
                    # Vary the dot intensity
                    if cpu_data['percent'] > 75:
                        content.append("∙", style="red")
                    elif cpu_data['percent'] > 50:
                        content.append("∙", style="yellow")
                    else:
                        content.append("∙", style="green")
                else:
                    content.append("·", style="dim")
            content.append(" │\n", style="dim")

        content.append("      └", style="dim")
        content.append("─" * (cols + 2), style="dim")
        content.append("┘\n", style="dim")

        return content


class CPUPanel(Static):
    """CPU panel with per-core breakdown."""

    def __init__(self, hw: HardwareInfo, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.hw = hw

    def render(self) -> Text:
        """Render CPU panel with btop-style layout."""
        cpu_data = self.hw.get_cpu_usage_cost()
        per_core = cpu_data['per_core']

        content = Text()

        # Header with model name
        content.append(f"{'':>50}{self.hw.cpu_model}\n", style="bold cyan")

        # Overall CPU bar
        content.append(f"{'':>50}CPU ", style="bold cyan")
        bar_width = 40
        filled = int((cpu_data['percent'] / 100.0) * bar_width)

        # Color based on usage
        if cpu_data['percent'] < 50:
            color = "green"
        elif cpu_data['percent'] < 75:
            color = "yellow"
        else:
            color = "red"

        # Dot-based bar
        for i in range(bar_width):
            if i < filled:
                content.append("▓", style=color)
            else:
                content.append("░", style="dim")

        content.append(f" {cpu_data['percent']:>5.0f}%\n", style="bold")

        # Per-core display in two columns
        half = len(per_core) // 2
        for i in range(half):
            # Left core
            left_core = i
            left_usage = per_core[left_core]

            # Right core (if exists)
            right_core = i + half
            right_usage = per_core[right_core] if right_core < len(per_core) else 0

            # Left side
            content.append(f"{'':>50}C{left_core:<2} ", style="cyan")

            # Dot activity indicator for left
            dot_width = 20
            left_dots = int((left_usage / 100.0) * dot_width)
            if left_usage > 75:
                left_color = "red"
            elif left_usage > 50:
                left_color = "yellow"
            else:
                left_color = "green"

            for j in range(dot_width):
                if j < left_dots:
                    content.append(":", style=left_color)
                else:
                    content.append("·", style="dim")

            content.append(f" {left_usage:>4.0f}% ", style=left_color)

            # Right side
            if right_core < len(per_core):
                content.append(f"C{right_core:<2} ", style="cyan")

                # Dot activity indicator for right
                right_dots = int((right_usage / 100.0) * dot_width)
                if right_usage > 75:
                    right_color = "red"
                elif right_usage > 50:
                    right_color = "yellow"
                else:
                    right_color = "green"

                for j in range(dot_width):
                    if j < right_dots:
                        content.append(":", style=right_color)
                    else:
                        content.append("·", style="dim")

                content.append(f" {right_usage:>4.0f}%", style=right_color)

            content.append("\n")

        # Cost info
        content.append(f"\n{'':>50}", style="dim")
        content.append(f"💰 Hardware: ${cpu_data['used_cost']:.2f} / ${cpu_data['total_cost']:.2f}\n", style="yellow")
        content.append(f"{'':>50}", style="dim")
        content.append(f"⚡ Power: {cpu_data['power_watts']:.1f}W ", style="yellow")
        content.append(f"(${cpu_data['power_cost_per_sec']:.6f}/sec = ${cpu_data['power_cost_per_sec']*3600:.4f}/hr)\n", style="dim")

        return content


class MemoryPanel(Static):
    """Memory panel with btop-style bars."""

    def __init__(self, hw: HardwareInfo, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.hw = hw

    def _dot_bar(self, percent: float, width: int = 40) -> Text:
        """Create a dot-based progress bar."""
        filled = int((percent / 100.0) * width)

        if percent < 50:
            color = "green"
        elif percent < 75:
            color = "yellow"
        else:
            color = "red"

        bar = Text()
        for i in range(width):
            if i < filled:
                bar.append(":", style=color)
            else:
                bar.append("·", style="dim")
        return bar

    def render(self) -> Text:
        """Render memory panel."""
        mem_data = self.hw.get_memory_usage_cost()

        content = Text()
        content.append("mem\n", style="bold magenta")

        # Total
        content.append("Total:     ", style="dim")
        content.append(f"{mem_data['total_gb']:>6.1f} GiB\n", style="white")

        # Used
        content.append("Used:      ", style="dim")
        content.append(f"{mem_data['used_gb']:>6.1f} GiB ", style="white")
        content.append(f"{mem_data['percent']:>3.0f}%\n", style="yellow")
        content.append("           ")
        content.append(self._dot_bar(mem_data['percent']))
        content.append("\n")

        # Available
        content.append("Available: ", style="dim")
        content.append(f"{mem_data['available_gb']:>6.1f} GiB ", style="white")
        avail_pct = (mem_data['available_gb'] / mem_data['total_gb']) * 100
        content.append(f"{avail_pct:>3.0f}%\n", style="green")
        content.append("           ")
        content.append(self._dot_bar(avail_pct))
        content.append("\n")

        # Cost
        content.append("\n💰 Cost:   ", style="dim")
        content.append(f"${mem_data['used_cost']:.2f} / ${mem_data['total_cost']:.2f}\n", style="yellow")

        return content


class DiskPanel(Static):
    """Disk panel with btop-style bars."""

    def __init__(self, hw: HardwareInfo, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.hw = hw

    def _dot_bar(self, percent: float, width: int = 40) -> Text:
        """Create a dot-based progress bar."""
        filled = int((percent / 100.0) * width)

        if percent < 50:
            color = "green"
        elif percent < 75:
            color = "yellow"
        else:
            color = "red"

        bar = Text()
        for i in range(width):
            if i < filled:
                bar.append(":", style=color)
            else:
                bar.append("·", style="dim")
        return bar

    def render(self) -> Text:
        """Render disk panel."""
        disk_data = self.hw.get_disk_usage_cost()

        content = Text()
        content.append("disks\n", style="bold blue")

        # Total
        content.append("Total:     ", style="dim")
        content.append(f"{disk_data['total_gb']:>6.1f} GiB\n", style="white")

        # Used
        content.append("Used:      ", style="dim")
        content.append(f"{disk_data['used_gb']:>6.1f} GiB ", style="white")
        content.append(f"{disk_data['percent']:>3.0f}%\n", style="yellow")
        content.append("           ")
        content.append(self._dot_bar(disk_data['percent']))
        content.append("\n")

        # Free
        content.append("Free:      ", style="dim")
        content.append(f"{disk_data['free_gb']:>6.1f} GiB ", style="white")
        free_pct = 100 - disk_data['percent']
        content.append(f"{free_pct:>3.0f}%\n", style="green")
        content.append("           ")
        content.append(self._dot_bar(free_pct))
        content.append("\n")

        # Cost
        content.append("\n💰 Cost:   ", style="dim")
        content.append(f"${disk_data['used_cost']:.2f} / ${disk_data['total_cost']:.2f}\n", style="yellow")

        return content


class ProcessList(Static):
    """Scrollable process list with btop styling."""

    def __init__(self, hw: HardwareInfo, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.hw = hw
        self.table = DataTable(zebra_stripes=True)

    def on_mount(self) -> None:
        """Setup the data table."""
        self.table.add_columns("PID", "User", "Name", "Mem", "CPU%", "💰 HW$/s", "⚡ Power$/s", "Total$/s")
        self.table.cursor_type = "row"

    def compose(self) -> ComposeResult:
        """Compose the scrollable table."""
        yield self.table

    def update_processes(self) -> None:
        """Update process list."""
        processes = self.hw.get_top_processes(limit=100)

        # Clear and repopulate
        self.table.clear()

        for proc in processes:
            cost = proc['total_cost']
            user = proc.get('username', 'unknown')

            # Dot-based CPU indicator
            cpu_dots = int((proc['cpu_percent'] / 100.0) * 10)
            cpu_vis = "·" * (10 - cpu_dots) + ":" * cpu_dots if proc['cpu_percent'] > 0 else "·" * 10

            hw_cost = proc['cpu_cost'] + proc['mem_cost']

            self.table.add_row(
                str(proc['pid']),
                user[:8],
                proc['name'][:30],
                f"{proc['memory_mb']:.0f}M",
                f"{cpu_vis} {proc['cpu_percent']:>4.1f}",
                f"${hw_cost:.4f}",
                f"${proc['power_cost']:.6f}",
                f"${proc['total_cost']:.4f}",
                key=str(proc['pid'])
            )


class MTopApp(App):
    """mtop - Money-based resource monitor."""

    CSS = """
    Screen {
        layout: vertical;
    }

    #top-section {
        height: auto;
        layout: horizontal;
    }

    #cpu-viz {
        width: auto;
        height: auto;
    }

    #cpu-info {
        width: 1fr;
        height: auto;
    }

    #middle-section {
        height: auto;
        layout: horizontal;
    }

    #left-stats {
        width: 1fr;
        padding: 1 2;
    }

    #right-stats {
        width: 1fr;
        padding: 1 2;
    }

    #process-section {
        height: 1fr;
        padding: 1 2;
    }

    ProcessList {
        height: 100%;
    }

    DataTable {
        height: 100%;
    }

    DataTable > .datatable--header {
        text-style: bold;
    }
    """

    BINDINGS = [
        ("q", "quit", "Quit"),
        ("r", "refresh", "Refresh"),
    ]

    def __init__(self):
        super().__init__()
        self.hw = HardwareInfo()

    def compose(self) -> ComposeResult:
        """Compose the UI."""
        yield Header(show_clock=True)

        # Top section: CPU visualization and stats
        with Horizontal(id="top-section"):
            yield CPUVisualizer(self.hw, id="cpu-viz")
            yield CPUPanel(self.hw, id="cpu-info")

        # Middle section: Memory and Disk
        with Horizontal(id="middle-section"):
            with Vertical(id="left-stats"):
                yield MemoryPanel(self.hw)
            with Vertical(id="right-stats"):
                yield DiskPanel(self.hw)

        # Bottom: Process list
        with Container(id="process-section"):
            yield ProcessList(self.hw)

        yield Footer()

    def on_mount(self) -> None:
        """Start the update loop when app mounts."""
        self.set_interval(1.0, self.update_stats)
        self.title = "mtop"
        total = self.hw.get_total_hardware_cost()
        self.sub_title = f"💰 Hardware Value: ${total:,.2f}"

    def update_stats(self) -> None:
        """Update all statistics."""
        # Refresh all panels
        self.query_one("#cpu-viz", CPUVisualizer).refresh()
        self.query_one("#cpu-info", CPUPanel).refresh()
        self.query_one(MemoryPanel).refresh()
        self.query_one(DiskPanel).refresh()

        # Update process list
        self.query_one(ProcessList).update_processes()

    def action_refresh(self) -> None:
        """Force refresh all data."""
        self.update_stats()


def main():
    """Run the app."""
    app = MTopApp()
    app.run()


if __name__ == "__main__":
    main()
