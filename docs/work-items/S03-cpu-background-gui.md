# S03 CPU 백그라운드 실제 GUI

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R08,R11
- 담당/수정 범위: guardian_fault; 신규 scripts/cpu_background_probe.py 및 전용 validation/evidence
- 선행 작업: root CPU background 구현/정확 frozen binary READY; Linux GUI 실행 완료

## 사용자 결과

합성 다운로드 중 숨김과 같은 작업 복귀, 숨긴 채 완료 뒤 복귀, 중단 후 종료, 복귀 endpoint 준비 실패 시 안전한 안내와 기존 파일 보존을 실제 native CPU GUI에서 확인한다.

## 계약과 범위

ownedXvfb/privatebus/0700task XDG/temp 합성fixture만. 기존helpers read-only reuse, 새harness독점, 제품/Cargo/sharedharness수정금지. AT-SPI 실제role/bounds, 일반content와modal 전체viewport 범위분리. visiblefallback/userdesktop/clipboard/실제media/network/auth금지. worker pidfd/start와secondaryprocess 소유확인, bounded120sec/64KiBlog. 기존8h재시작·중단없음.

## 인수 조건

active-hidden: 원app/worker같은identity/heartbeat진행/windowunmapped→secondaryrestore0/windowviewable/StopExitcleanup. complete-hidden: hidden Downloaded/원PIDrestore/UI완료/no다른Attempt. stop-exit: modal 실제선택/primary0/worker종료. unavailable: freshfixtureactivation.sock sentinel identity/hash보존/backgrounddisabled안내/return또는StopExit가능. 각4cases JSON/PNG/SQLite/cleanup/sourcebeforeafter 증거, 실패원인분리/미완료PASS미주장.

## 검증과 handoff

실제 4개 기능 동작 검증 완료. 최초 unavailable 접근성 state assertion 실패와 제한 실제 클릭 대조를 모두 보존했다. 비활성 버튼의 AT-SPI Enabled/Sensitive 불일치는 미해결이며 기능 PASS와 구분한다. 검증 명령·원시 경로·정확 hash·cleanup은 [전용 검증](../validation/cpu-background-validation.md)에 기록했다. Windows/macOS 백그라운드 CPU GUI는 미검증.
