# S10 통합 Windows CPU CLI

2026-10-03 KST. 첫 실행 Clippy6.55초exit0, test9.08초exit101. 기존 accessible.rs Fixture::new create_dir(642줄)에서 Os183 AlreadyExists로 active_worker_rejects_second_worker_without_ui_join이실패했다. fixture pid+SystemTime nanoseconds 패턴은 Windows 시계해상도에따라병렬테스트경로충돌가능성이있다. 같은binary의신규진단7개회귀는통과했지만전체workspace시험성공으로확대하지않는다.

실패입력/정확argv/원시로그는windows-cpu-diagnostics/{clippy,test}-inputs.json/.log에보존했다. sourcebefore/after unchangedTrue. 빌드는시험실패즉시중단하여실행하지않았다. 제품/fixture소스수정은통합담당합의전수행하지않았다. 후속collision-safe fixture수정후전체세검사와runner/source/binarySHA를재확인한다.
