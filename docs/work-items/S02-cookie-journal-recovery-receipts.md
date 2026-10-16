# S02: journal의 완료 reader receipt 복구

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — cookies/reader_journal.rs 및 이 문서
- 선행 작업: root의 native4 source 동결 해제, anchored journal 및 durable receipt

## 사용자 결과

같은 부팅 복구의 호출자가 실제 종료 receipt를 읽어 남은 Bound reader를 durable Resolved로 바꿀 수 있게 한다. 제품 cookie recovery 연결은 root 담당이며 이 모듈만으로 전체 복구 완료를 주장하지 않는다.

## 계약과 범위

`recover_completed_readers(&mut self) -> Result<bool, JournalError>`는 directory/record/receipt native ID 및 record bytes를 먼저 검증한다. Intent가 있으면 false로 보존한다. 원래 Bound 또는 Resolved 슬롯이 하나 이상이어야 한다. 빈 journal은 false이며 legacy ungated reader를 같은 부팅에서 삭제할 근거로 사용하지 않는다.

Bound 슬롯은 원본 파일의 실제 256바이트 receipt를 읽고 capability와 native identity 전체를 정확히 대조한 뒤 resolve의 파일·directory sync를 수행한다. Resolved도 실제 receipt를 재검증한다. proof가 비어 있거나 부분/불일치이면 false로 보존한다. 경로 entry 소실·native ID 교체·mode/UID/link 변경은 ownership 오류이며 missing proof와 구분한다. 최종 슬롯이 전부 Empty/Resolved일 때만 true다. cleanup/unlink는 수행하지 않는다.

## 인수 조건

Bound의 실제 receipt에서 resolve 및 reopen 후 검증, 빈 journal과 Intent 보존, zero/partial/mismatch receipt의 bytes 보존, 교체된 native entry의 오류 및 원본 보존을 시험한다. schema와 슬롯 외부 노출은 변경하지 않는다.

## 검증과 handoff

root native4 compile source 동결 동안 제품 소스 수정은 보류한다. 별도 임시 patch를 준비하고 root 해제 메시지 뒤 적용한다. Cargo/native 실행은 root 담당이다.

## 구현 동결

root의 native4 실제 4PASS 및 immutable 증거 보존 완료 메시지 이후 준비한 patch를 적용했다. API는 최초 identity/record 검증과 모든 receipt preflight 뒤 기존 resolve의 durable commit을 재사용한다. 기존 슬롯/schema/cleanup/unlink는 변경하지 않았다.

회귀 3개를 추가했다. 실제 synthetic Bound receipt 저장·sync→resolve→reopen true, 빈 journal 및 receipt가 있어도 Intent false, zero/partial/mismatch 및 receipt native entry 교체의 record/receipt bytes 보존을 다룬다. 이 시험은 native manager proof가 아니라 journal proof 소비 계약 시험이다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/src/cookies/reader_journal.rs` 실제 exit 0. 소스 SHA256 `8c0af07996eac613e7b16c1ba137d0c132c7ac910724fcfcc41bc8580d1855fa`를 root에 동결 전달했다. Cargo와 native 실행은 하지 않았다. root가 cookies.rs startup recovery에 연결하고 실제 통합 시험한다.
