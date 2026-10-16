# S03: CPU 도구·이력·접근성 통합 검증

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R08, R10, R11
- 담당/수정 범위: root; accessible.rs/두 entry/managed.rs API, Cargo.toml/Cargo.lock/vendor patch 및 본 문서·전용 통합 증거
- 선행 작업: S04-cpu-managed-tools, S03-cpu-history, S00-atspi-disabled-state

## 사용자 결과

CPU 후보에서 관리 도구와 이력을 선택하고 실제 검증된 경로/원래 옵션을 적용한다. 닫기에는 모든 소유 worker 종료 ack를 기다린다. 비활성 버튼의 보조기술 상태를 바로잡는다.

## 계약과 범위

원래 engine를 사용하고 새 DB schema/dependency version은 없다. local accesskit_atspi_common 0.14.2 최소 patch와 실제consumerTree 회귀, Linux 격리 GUI, Windows nativeCLI 범위를 구분한다. 최초 cargo offline 재해석의 불필요한 dependency edge 변경은 원래 lock으로 복구하여 registry source/checksum 두 줄 제거만 남겼다. vendor source 약141KiB, 런타임 기능 추가 없음.

## 인수 조건

공통4명령/CPUfeature 및 Linux GUI 도구·이력·비활성 상태·숨김 복귀 독립 대조. Windows GNU current input manifest/실제 시험/로그/desktop unchanged 대조. 실제 WindowsGUI/macOS/설치/전체P0로 확대하지 않는다.

## 검증과 handoff

consumerTree1test/8fixtures, bounded engine history3tests, CPU59tests/alltargetCPUclippy/release 성공. LinuxCPU SHA522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd; GUI 및 공통/Windows 통합검증 진행 중. 최초 type/clippy 실패는 수정 후 재검증했다.

## 고정 snapshot 통합 결과

522e snapshot 기본199/CPU312 PASS(각5ignored), Windows GNU228PASS/1ignored, fmt/default 및 CPUclippy/release PASS. 254개 native input을 root가 대조했다. tools/history raw JSON·SQLite·PNG와 AT-SPI 실제 bits/SQLite를 root가 독립 검토했다. 비용은 별도 idle 측정에 남겼으며 이후 callback/metrics source에 이 snapshot PASS를 확대하지 않는다. 설치네트워크/OSGUI/전체P0는 남는다.
