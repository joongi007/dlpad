# 공통 에이전트 작업 계약

이 문서는 특정 도구에 종속되지 않는다. Codex, Claude Code, 기타 IDE/CLI 에이전트와 사람이 동일한 기준을 사용한다. 자동으로 읽지 않는 도구에는 작업 시작 시 이 파일을 명시적으로 제공한다.

## 먼저 읽기

README.md → PRODUCT.md → CONTEXT.md → 관련 요구사항 → 관련 ADR 순으로 읽는다. UI는 DESIGN.md, 프로세스/DB는 docs/ARCHITECTURE.md, 도구/인증은 docs/INTEGRATIONS.md를 추가로 읽는다. 문서가 충돌하면 사용자 지시와 채택된 ADR을 우선하고 충돌을 수정한다. Proposed 결정을 사용자 확정으로 표현하지 않는다.

## 구현 규칙

- 앱 UI는 Rust 네이티브. WebView/Electron/Tauri UI와 localhost UI 서버를 도입하지 않는다.
- core 상태/정책은 UI·프로세스·DB에서 분리한다. 긴 I/O를 UI thread에서 수행하지 않는다.
- shell 문자열 실행 금지. 외부 도구는 argv, 소유 프로세스 트리, 취소·타임아웃과 함께 관리한다.
- 외부 설치, 다운로드 파일, 자격 증명을 앱 소유 cache처럼 삭제하지 않는다.
- 진행률·로그·썸네일·scan queue에는 상한을 둔다. stream 전체를 메모리에 적재하지 않는다.
- 자동 GUI 시험은 사용자 데스크톱과 분리된 가상 화면에서 실행한다. 가상 화면 실패 시 visible desktop으로 fallback하지 않으며, `--desktop` 시험은 사용자가 명시적으로 요청한 경우에만 실행한다.
- Windows/macOS/Linux 경로·시그널·트레이·IME 차이를 고려한다. 확인하지 않은 OS를 검증 완료로 보고하지 않는다.
- 버전·feature·의존성 추가 시 크기와 런타임 비용을 설명한다. Cargo.lock과 정확한 toolchain을 커밋한다.
- 새 도메인 용어·상태 전이는 CONTEXT와 ADR에 반영한다. UI 문자열은 중앙 catalog로 관리한다.

## 작업 방식

작업마다 docs/agents/WORK_ITEM_TEMPLATE.md로 범위, 요구사항 ID, 수정 파일, 인수 조건을 기록한다. 한 작업은 사용자에게 검증 가능한 수직 기능 단위로 끝낸다. 범위 밖 개편은 후속 작업으로 남긴다.

여러 에이전트를 쓰는 경우에는 명시적으로 파일/인터페이스 담당을 나누고 같은 파일의 동시 수정을 피한다. 이 문서는 자동 위임을 요구하지 않는다. git이 준비된 이후 분리 branch/worktree를 사용한다. 공유 계약·schema 변경은 담당자와 합의하고 통합 담당자가 전체 검증한다.

작업 완료 보고: 바뀐 사용자 동작, 수정 파일, 실제 실행한 명령과 결과, 미검증 사항, 남은 결정. 성공하지 않은 테스트를 성공으로 쓰거나 구현 계획을 구현 완료로 쓰지 않는다.

## 공통 검증

소스 checkout/CI: `python3 scripts/check_docs.py --source-only`. 원본 로컬 증거까지 있는 작업 폴더: `python3 scripts/check_docs.py`.

Rust workspace 도입 후: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked`.

상태 복구·취소·파일 삭제·인증·옵션 파싱에는 실패 경로 테스트가 필요하다. 단순 문구 수정에는 구현을 그대로 복제한 테스트를 추가하지 않는다. 시각·OS 동작 검증은 단위 테스트 통과로 대체하지 않는다.

## Agent skills

공통 설정은 docs/agents/README.md에 있다. 로컬 작업 추적은 docs/work-items/를 사용한다. GitHub 원격은 https://github.com/joongi007/dlpad 이며 issue tracker는 로컬 docs/work-items/를 유지한다. CONTEXT.md와 docs/adr/가 도메인/결정 기준이다. 선택적 skill이 없어도 작업 가능해야 하며 특정 사용자 홈 경로를 의존성으로 추가하지 않는다.

### Issue tracker

기존 로컬 docs/work-items/ 계약을 유지한다. [설정](docs/agents/issue-tracker.md).

### Triage labels

기존 작업 상태/분류와 skill의 보조 역할을 구분한다. [설정](docs/agents/triage-labels.md).

### Domain docs

CONTEXT.md와 docs/adr/의 단일 문맥을 사용한다. [읽기 계약](docs/agents/domain.md).
