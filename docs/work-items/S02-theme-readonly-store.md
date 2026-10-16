# S02: 읽기 전용 화면 설정 조회

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: 주 담당; crates/engine/src/store.rs, 본 문서. settings 화면은 library_identity 전용.
- 선행 작업: 기존 Store와 theme 설정 계약

## 사용자 결과

화면 설정을 읽을 때 없는 DB를 생성하거나 기존 DB를 마이그레이션하지 않는다. 과대 설정은 오류로 구별하며 기존 데이터를 보존한다.

## 계약과 범위

Store::read_setting_readonly(path,key,max_bytes) 추가. READ_ONLY/NO_MUTEX/NOFOLLOW, timeout 5초. SQL에서 byte length를 검사한 뒤 제한 이내 문자열만 반환. schema와 Cargo 변경 없음. 최종 파일 symlink 거부이며 상위 경로 검사는 호출자 책임이다.

## 인수 조건

일반 설정/없는 key, Unicode byte 제한, 없는 파일, schema 없는 DB, 손상 파일, symlink, exclusive lock 오류와 원본 보존을 실제 임시 DB로 검증한다.

## 검증과 handoff

`cargo test -p dlpad-engine --locked store::tests -- --nocapture` exit0: 7 passed/0 failed, 5.15초(잠금 timeout 포함). 새 API의 실패 경로 4시험과 기존 3시험을 실제 실행했다. GUI와 OS별 동작은 이 API 시험으로 완료 주장하지 않는다. SQLite 자체가 외부 WAL DB의 sidecar를 취급할 수 있어 모든 임의 DB의 sidecar 무생성을 주장하지 않는다.
