# S02: 실행 전용 session cookie adapter

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R07, R08, R13
- 담당/수정 범위: metrics_recovery — crates/engine/src/download.rs, 이 문서만
- 선행 작업: stress_recovery SessionCookieLease::validate_for_tool; root native UI/catalog/shared integration

## 사용자 결과

명시 선택한 세션 lease를 metadata/download 실행에만 적용한다. 인증 raw 로그와 metadata 자유 문자열은 외부 결과에서 생략하며, artifact/subtitle identity 검증은 원래 내부 증거로 유지한다. UI 연결 완료는 이 작업 범위 밖이다.

## 계약과 범위

DownloadSession은 cancel/pause/progress와 borrowed cookie lease를 묶는다. 기존 API는 그대로 유지한다. snapshot/options/schema에는 cookie path/content를 넣지 않는다. version/FFmpeg version argv에도 넣지 않는다. 각 인증 tool 단계 직전에 lease를 재검증한다. 로그 생략은 내부 artifact/subtitle parsing 뒤 수행하며 outcome/exit/truncated를 보존한다. Json/Io/DB 오류의 자유 텍스트도 인증 반환에서는 고정 오류로 대체한다. metadata 반환은 duration만 유지한다. lease cleanup은 UI/session 소유자가 모든 종료 acknowledgment 후 담당한다. 새 dependency/thread/persisted schema 없음.

## 인수 조건

합성 fixture로 metadata/download cookie argv 동일·version 미포함, 결과/오류/DB/snapshot의 secret 배제, subtitle 내부 ID 검증 유지, cleanup/source sentinel 보존, 취소/metadata 오류/실행 오류/재개를 검증한다. 실제 credential/사이트/GUI는 이 adapter 시험에 포함하지 않는다. 기존 unauthenticated 회귀를 유지한다.

## 검증과 handoff

Cargo는 root 직렬 담당이며 현재 작성·검증 전이다. source freeze 뒤 root와 독립 시험 담당에게 API/실행 계약을 전달한다.


## 최초 adapter 동결

기존 API를 유지하고 public DownloadSession(cancel/pause/progress/cookies), execute_with_session, resume_with_session을 추가했다. metadata/download 직전에 validate_for_tool()을 호출하고 `--cookies`와 tool-only path를 argv에 넣는다. version 단계에는 넣지 않는다. 성공과 오류는 외부 반환 전에 raw 로그를 `[authenticated tool output omitted]`로 대체하고 outcome/exit/truncated를 유지한다. 내부 metadata ID는 subtitle 검증까지 유지하고 반환은 duration만 남긴다. authenticated Json/Io/DB 오류의 자유 텍스트는 고정 오류로 대체한다. 새 pure 실패 unit2를 작성했다.

source SHA256 `7755ad6fe1a0d5d80d42d298e62da54474348de0c1cf3e62969b30660e0821a0`, rustfmt exit0. Cargo 및 실행 시험은 root와 별도 integration 담당이 수행해야 하며 아직 통과를 주장하지 않는다. UI/root catalog 연결은 root 소유다.

## 미확인 process 정리와 lease 수명

process::run*가 reaper transfer 후 반환하는 두 명시 오류(`guardian startup failed; child cleanup deferred`, `guardian startup failed; child cleanup could not be confirmed`) 및 output drain 중단의 protection-incomplete 오류는 actual child 종료 acknowledgment를 뜻하지 않는다. 이 경우 synchronous borrow 종료 뒤 lease를 일반 Drop하면 private copy가 아직 필요한 실행과 수명이 분리된다. 현재 자유 오류 생략은 이 소유권 정보를 지워선 안 된다. root에 typed process OwnershipCleanupError와 download SessionCleanupDeferred 구분 및 UI/session의 owned-copy 보존을 제안했다. 실제 구현과 시험 전 vertical 완료로 판정하지 않는다.

또한 native user_service의 cleanup failure가 launcher exit126+stderr로만 전달되는 경로는 nonzero tool exit와 확실히 구별되지 않는다. arbitrary log/exit-code 패턴으로 cleanup 미확인을 추정하면 안 되며, bounded terminal ownership acknowledgment 등 공유 process 계약이 필요하다. 이 한계는 lease borrow가 정상 두 실행 전체에 유지된다는 사실만으로 해결되지 않는다.


## upfront 거절 및 typed 정리 미확인 보완

root 실제 integration 최초4PASS/1FAIL에서 cleaned/replaced lease에도 version 호출이 발생하는 결함을 확인했다. execute_observed 진입 시 lease를 먼저 검증하여 job attempt 기록과 version/tool 실행 이전에 거절한다. metadata/download 각각 직전 검증도 유지했다. 시험 기대를 완화하지 않았다.

root가 process adapter의 명시3경로에 OwnershipCleanupError(StartupDeferred/StartupUnconfirmed/OutputDrainInterrupted)를 추가했다. download의 From<io::Error>는 cleanup_is_unconfirmed downcast를 먼저 확인하여 SessionCleanupDeferred로 바꾸고, 인증 결과 생략은 이 variant를 그대로 유지한다. 고정 Display는 `process cleanup unconfirmed; session copy must be retained`이며 UI/session 소유자가 복사본을 이미 보존했다고 주장하지 않는다. 동일한 free-text 메시지만 가진 일반 I/O 오류는 이 variant로 분류하지 않는다. typed3variant→download error→omission 유지와 spoof 문자열 거절 unit을 추가했다.

최신 download source SHA256 `57f27c44c5f0fb980242d9047c5e93f3a60868f83eba3047b1627f7f67ce2b71`, rustfmt exit0, 신규 unit3개 작성. Cargo는 아직 자체 실행하지 않았으며 root 통합검증 대상이다. UI가 이 variant에서 owned lease를 보존하는 동작과 다음 시작 recovery는 root/stress 소유다.

이 typed adapter는 명시된 process3경로만 판정한다. user_service 내부 cleanup 미확인이 launcher exit126/stderr로만 종료되는 terminal acknowledgment 한계는 남는다. 임의 tool exit126이나 로그 문자열을 정리 미확인으로 추정하지 않는다. 이 경계의 별도 소유성 증거/terminal 계약과 세션 보존 인수를 완료하기 전 전체 인증 기능이 완료되었다고 표현하지 않는다.
