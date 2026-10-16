# Windows GNU CPU 도구·이력 통합 CLI 검증

Linux CPU SHA `522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd`와 같은 source snapshot을 Windows GNU native에서 검증했다. 후속 activation wake callback 변경은 이 결과에 포함하지 않는다.

native clippy/test/release exit0, 228 passed/0 failed/1 ignored. 각 명령의 254개 입력 manifest와 실행 후 프로젝트 파일을 root가 대조했다. 입력 desktop은 세 명령 모두 unchanged다. Windows binary SHA `ba04fbbe4897fc56b5eef5ce4780d687c80f33e3dba7dd12fd299c15ffe473ca`.

[원시 통합 결과](windows-cpu-parity/result.json)와 같은 폴더의 로그·입력 manifest를 따른다. 실제 제품 GUI·IME·clipboard·MSVC·macOS 검증은 아니다. Unix 전용 bounded-history 실패 경로 시험은 Windows에서 실행되지 않았다.
