"""Offline tests for the executable architecture-boundary matchers."""

import pytest

from scripts.check_architecture_boundaries import (
    find_forbidden_db_refs,
    find_forbidden_event_calls,
    find_python_effect_exports,
    find_python_execution_commands,
)


pytestmark = [pytest.mark.offline]


def test_python_execution_command_allowlist_rejects_novel_command():
    source = """
    generate_handler![
        commands::python::py_get_live_info,
        commands::python::py_download_music,
        commands::python::py_download_extra,
    ]
    """

    assert find_python_execution_commands(source) == {"py_download_extra"}


def test_python_execution_command_allowlist_keeps_music_exception():
    source = """
    py_command_str!(py_get_live_info, crate::python::handler::get_live_info, LiveInfoResult);
    pub async fn py_download_music() {}
    """

    assert find_python_execution_commands(source) == set()


@pytest.mark.parametrize(
    "source, expected",
    [
        ("async def handle_one_video(self, url): pass", {"handle_one_video"}),
        ("def start_live_record(url): pass", {"start_live_record"}),
        ("def download_video_v2(url): pass", {"download_video_v2"}),
        ("def _make_downloader(self): pass", {"_make_downloader"}),
        ("async def handle_download_music(self, url): pass", set()),
    ],
)
def test_python_effect_export_matcher(source, expected):
    assert find_python_effect_exports(source) == expected


def test_db_and_event_matchers_use_the_same_rules_as_production_guard():
    assert find_forbidden_db_refs('db.execute("INSERT INTO live_records ...")') == {
        "live_records"
    }
    assert find_forbidden_event_calls('emit("task-update", payload)') == {"emit("}
    assert find_forbidden_event_calls("broadcast_task_update(task)") == {"broadcast_"}
