# S04: FFmpeg·ffprobe 정확 version 선택 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R05, R11
- 담당/수정 범위: metrics_recovery — /tmp/dlpad-ffmpeg-version.rs 순수 초안 및 이 문서만; root 실제 worker/선택/공급 schema 담당
- 선행 작업: 공식 v2 inactive 세트 설치와 원 component hash

## 실제 출력과 parser

기존 ffmpeg-supply-logs/ffmpeg-version.txt는12줄2379B, ffprobe-version.txt는10줄2354B다. 첫 줄 exact token은 n9.0.2-17-g2a571b6068-20260930이며 각각 'ffmpeg version ... Copyright (c) 2000-2026 the FFmpeg developers', ffprobe는2007-2026이다. ffmpeg 기록에는 마지막 빈 줄과 'Exiting with exit code 0'도 있으므로 기존 merged log와 실제 stdout 분리는 root 새 ProcessSpec 실증에서 확인한다.

순수 verified_ffmpeg_version(stdout,expected_tool,expected_version)->bool 초안은 tool ffmpeg|ffprobe, 전체64KiB/≤64줄/줄4096B/printableASCII, exact 첫 줄 token+Copyright 구조와4digit-year range를 요구한다. expected version도128B bounded 단일 ASCII token이다. CRLF 허용, 이름·버전substring·앞banner·overflow·invalidbytes/control을 거절하는3시험을 작성했다. 후속 줄 내용은 version의 유일 증거로 취급하지 않으며 license/buildconfig 채택은 별도 proof다. 이것은 tool exit/truncation/원hash 검증을 수행하는 worker를 대신하지 않는다.

## 실제 선택 worker 인수

root가 ['-version'] exact argv를 두 원component에 owned ProcessSpec으로 실행한다. 원directory/main/ffprobe pins, cancel/동일boundeddeadline/출력상한과 zeroexit/nontruncated를 확인하고 parser 성공·전체posthash 뒤에만 active DB를 변경한다. version 불일치/nonzero/timeout/cancel/원component 교체/foreign/license 미완비는 prioractive 보존이다. 두 component가 같은 버전이라는 것과 공급 trust/원source/notice 의무는 서로 다른 gate다.

## 검증과 handoff

기존 원시 version 파일 읽기 및 /tmp 순수 parser 초안/개별rustfmt만 수행했다. 프로젝트 module/Rust/Cargo/실제tool 실행은 하지 않았다. root 통합과 실제 official stdout/worker 검증을 기다린다. 공급 채택·관리 선택 완료로 표시하지 않는다.

## 실제 구현 진행

root가 crates/engine/src/tools/ffmpeg_version.rs와 tools.rs 등록, examples/install_managed_archive.rs의 tool별 local-verified dispatch를 추가했다. 두 실행에 동일10초 deadline을 적용하고 main/probe 원 file pin 및 directory pin을 각 실행 뒤 확인한다. 전체 component 재검증 후 active transaction을 commit한다. metadata capture 공통 상한은8MiB이며 parser 및 log 상한64KiB와 다르다. 실제 공급 license/source/notice 채택은 별도이며 이 API의 버전 확인으로 충족하지 않는다. Linux 순수 parser 시험과 실제 process 실패 회귀를 실행 중이며 성공 결과는 실제 종료 뒤 기록한다. 하위 에이전트 담당은 별도 tests/ffmpeg_version_selection.rs 및 대응 work-item으로 한정한다.

### 첫 검증 후 보정

Linux parser3시험과 실제 합성 선택1시험(성공 전환 및5실패 case)이 PASS. 첫 all-target Clippy는 items_after_test_module로 exit101이어서 test module을 끝으로 이동했다. 재검증 및 공식 바이너리 실행은 후속 확인한다.

### 실제 검증 결과

`cargo test -p dlpad-engine --lib tools::ffmpeg_version --locked` 3PASS, `cargo test -p dlpad-engine --test ffmpeg_version_selection --locked` 실제 합성 process1시험 PASS, 경고 수정 후 `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0, release example build exit0. 공식 원 TAR/XZ를 임시 관리 root에 설치하고 실제 ffmpeg/ffprobe -version 확인 후 active 전환 및 active 제거 거절을 확인했다. [원시 결과](../validation/evidence/ffmpeg-v2-verified-selection-first/result.json) exit0/PASS, source 전후 SHA 동일, install36778ms·선택 및 제거 거절2383ms. 공급 채택 false. 이후 소유 tempfile 종료 정리는 active 사용자 제거와 다르다. 실제 Windows/macOS 버전 실행, 전체 최신 workspace와 관리 GUI는 미검증이다.

### 후속 전체 검증 진행

Windows GNU CPU workspace fmt/clippy/test/release4명령 exit0. ffmpeg-selection-windows-workspace-first/root-review.json에서 주 담당이 최종380 source SHA를 현재 파일과 재대조해 불일치0을 확인했다. Windows 실제 FFmpeg runtime/GUI 또는 macOS 검증을 뜻하지 않는다. Linux 첫 collector는 Cargo PATH 누락으로 도구 시작 전 terminal exit1이므로 원시 보존; 정확 프로젝트 환경 지정 env-retry collector를 새 evidence에서 실행 중이다. 해시·DB 취소 경계 보완은 source freeze 해제 뒤 수행한다.
