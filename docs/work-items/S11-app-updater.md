# S11: 기능 완성과 Git·CI/CD 이후 앱 updater

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R05, R08, R10, R11
- 담당/수정 범위: Root 공유 Cargo/lock/lib·UI·통합, updater_audit isolated worktree의 engine/app_update metadata·staging, update_transport /tmp의 HTTP adapter. 파일 동시 수정 없음
- 선행 작업: 사용자 요청 기능 완성·검증, 정상 Git 및 사용자 저장소에 반영, CI/CD 연결·실제 성공

## 사용자 결과

2026-10-03 사용자가 기능을 모두 만든 뒤 Git과 CI/CD에 올리고, 그 이후 앱 updater를 만들도록 순서를 지정했다. 이 작업은 앱 자체 업데이트이며 외부 사용자 도구 설치를 임의로 덮어쓰는 권한이 아니다.

## 계약과 범위

초기 기록 이후 2026-10-18에 구현을 시작했다. 앞 단계의 버그 수정·실패 경로·OS 회귀 검증을 유지한다. 업데이트 채널·서명 metadata·배포 대상·OS별 교체/rollback 정책은 실제 패키지/CI 결과를 근거로 구체화한다. 기존 .git을 삭제/재구성하거나 저장소 주소·공개 배포를 추측하지 않는다.

## 인수 조건

선행 기능 인수와 Git/CI/CD 실제 성공을 확인한 뒤 구현 착수. 정상 업데이트·손상/서명 오류 거절·중단/설치 실패 복구·기존 버전 유지·사용자 미디어/외부 도구 보존을 OS별 실제 시험으로 확인한다. 구체적 배포·권한 조건은 구현 전에 기록한다.

## 검증과 handoff

사용자 후속 요청을 기록했으며 updater 구현/배포/성공 검증은 아직 수행하지 않았다. 현재 정상 Git 저장소가 없고 P0·전체 기능 게이트가 미완료이므로 선행 작업 완료 전이다.

2026-10-11 준비 범위: scripts/validate_update_candidate.py는 최대64KiB의 중복 없는 JSON, 새 버전/channel/WindowsMSVC target, allowlist HTTPS host와 credential/query/fragment 금지, SHA256·256MiB package 상한 및 unsigned release_ready=false를 검사한다. root 정책 회귀3개 통과. 이는 다운로드·서명 검증·실제 교체·rollback 구현이 아니다. 별도 source Git bundle 도구와 Windows 우선 CI 구성을 준비했으며 외부 저장소 URL·실제 Actions 성공은 남는다.

## 2026-10-18 사용자 지정 실행 순서 갱신

Git 원격은 joongi007/dlpad이며 실제 Windows CI run39040643416이 전체 성공했다. 사용자 지정 순서는 소개 페이지→설치 ZIP/포터블 ZIP 자동 릴리스→앱 updater다. 위 Git 미준비 기록은 2026-10-03~11 당시 상태이며 현재 조건이 아니다. 지금은 Root가 기존 계약/코드를 감사하며 UI·공유 schema·Cargo/lock과 서명 trust root를 조정한다. 다운로드 도구 updater와 앱 updater를 구분한다. 공개 배포·비밀의 외부 전송은 별도 명시 범위로 유지하고 구현·합성 fixture·검증 가능한 초안은 계속 준비한다.

## 2026-10-18 구현과 Root 검증

서명 후보 검증과 Windows owned 저장을 engine app_update 모듈로 통합했다. 기존 TLS 경로의 ring0.17.14를 직접 재사용하며 Cargo.lock은 engine 의존성 한 줄만 추가하고 패키지 버전/개수는 추가하지 않았다. idle worker/network/모델 로딩을 추가하지 않으며 사용 시64KiB 고정 버퍼·최대256MiB 패키지·metadata64KiB다. Windows 게시 전후 두 번 hash하므로 패키지 크기2배의 로컬 read I/O가 들고 중간 전체 메모리 적재는 없다. 최종 PE 크기는 hosted 산출물 측정 대상이다.

Root actual Windows: `cargo clippy -p dlpad-engine --all-targets --locked --offline -- -D warnings` exit0; `cargo test -p dlpad-engine --lib --locked --offline app_update -- --test-threads=1` 14PASS/0FAIL. 명령은 프로젝트 소유 Windows private host/원 Job과 기존 pinned MSVC 준비 환경에서 실행했다. `.local-tools/update-integration-20261018/native-checks-v1/native-tests.log`의 원 stdout/종료 facts를 Root가 직접 읽었다. 원 Job natural-zero/final-zero=True, forced=False, input-desktop-unchanged=True, source_unchanged=True. 실제 모든14테스트명과 성공을 확인했으며 하위 에이전트 보고만으로 집계하지 않았다.

Linux Root 통합 verifier8+미지원1=9PASS. 기존 media_preview.rs:591 Linux unused_mut 경고가 있어 Linux strict workspace 성공을 주장하지 않는다. Windows strict engine lint는 통과했다. 전체 workspace CI와 HTTP/signing/네이티브 updater UI·다운로드·설치 전환·실패 복구는 남는다. [서명·권한 경계](../adr/0012-signed-app-update-candidates.md), [하위 감사/구현](S11-updater-audit-20261018.md).

## 2026-10-18 HTTP·native 화면·전체 Windows 통합

기존 ureq3.4.2로 공식 HTTPS metadata/signature/package adapter를 통합했다. 인증/쿠키/proxy0, redirect 목적지 요청 전 검증·최대4회, header16KiB, metadata64KiB, 서명128hex, package signed-size+1·절대deadline·cooperative cancel이다. 공개 GitHub 버전 조회는별도 read-only discovery이며 서명/설치 권한이 아니다. 설정 → 앱 업데이트는 현재 Cargo 버전·안정/미리보기·명시적 조회/중단·오류·결과 링크를 제공한다. idle worker/network0, result queue1/owned thread1, 모든탭 global poll/CloseWait ACK에 연결했고 다운로드 busy 표시에서 제외했다. 자동 설치 미제공을 중앙 catalog로 안내한다.

Root 실제 Windows 최종 명령은 `cargo clippy --workspace --all-targets --features dlpad/cpu --locked --offline -- -D warnings`, `cargo test --workspace --features dlpad/cpu --locked --offline`, `cargo build --workspace --features dlpad/cpu --release --locked --offline`다. guard 선행 build 포함4명령 exit0, 원 stdout43 harness 집계1729PASS/0FAIL/43ignored 중 filtered child1이 포함되어 주 harness1728+child1로 구분한다. `.local-tools/update-integration-20261018/native-full-v1/native-tests.log`를 Root가 직접 읽었고 natural-zero/final-zero=True, forced=False, final-active0, private-desktop-closed/input-desktop-unchanged/source-unchanged=True를 확인했다. 경과396.8초다. 새actualCPU10,420,736B, SHA25673c661d45be9b3b373ccbec461c8a74e261e51e818bf7c5b2ae4aca448012e41. 이전v37CPU 대비50,176B 증가이며 전체 변경 비용이지 ring 단독 비용 측정이 아니다. WindowsCPU/installer actual PE는 OS DLL만 사용하는 runtime 검사를 통과했다. clean machine 인증은 아니다.

Root Linux targeted `cargo test -p dlpad-engine --lib --locked --offline app_update` 27PASS(메타데이터9/HTTP13/discovery4/Windows 저장미지원1). Windows에서만 저장6회귀를 실제 실행했다. OpenSSL3.0.2 실제 signer의 public-only fixture를 ring이 검증하고 변조는 거절했다. 원fixture private key/ZIP은 임시 디렉터리에서만 사용했고 소스에는 포함하지 않았다. 운영 신뢰키·실제 공개 업데이트·실행 중 앱 교체·rollback은 아직 미완료다.
