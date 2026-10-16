# S02: 실제 Windows helper receipt의 제품 회수

- 상태: in-progress
- 분류: integration
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — windows/test_fixture.rs, windows_helper_launch.rs, 이 문서
- 선행 작업: 제품 import marker/journal/publicrecover, 실제 독립 helper

## 사용자 결과

제품 import의 원 journal에서 helper가 실제 도구를 실행·정리하고 생성한 receipt로 public startup recovery를 수행한다. 합성 temp/source만 사용한다.

## 계약과 범위

Test-only ProductFixture를 통해 private 원 journal handoff하며 실제 launcher/entry/runtime를 그대로 사용한다. 제품 adapter gatedreader 연결이나 actual appkill을 이 테스트로 대체하지 않는다.

## 인수 조건

Original native creation→durable Bound→helper lock ACK→Begin→Terminal0/native exit→actual256 receipt→app defer→publicrecover recovered1과 source/foreign보존. 사본과 원owner만 삭제하고 secondrecover recovered0.

## 검증과 handoff

구현전상태. native helper 선빌드 필요.

## 실제 native 결과

first Clippy/debugguardbuild exit0, integration test exit101/recovered0: 부모가 original receipt writer를 유지해 DELETE disposition 후 namespace 제거가 지연됐다. first 실패 자료 보존. root가 launch의 exactcap duplicate 전달 직후 parent writer drop을 명시한 후 second exact native test1 PASS/151 filtered/exit0. source_unchanged/input_desktop_unchanged true와 raw log를 root가 직접 확인했다. 실제 product import→별도helper nativecreation/Bound/ACK/Begin→native tool0/Terminal0/receipt256→appdefer→publicrecover recovered1/source와foreign보존/secondrecover0를 검증했다. 미래 product gatedreader도 부모 original writer를 handoff 직후 닫아야 하며 별도쓰기권한/미확인 livehandle을 남기면 cleanup 성공을 주장하지 않는다. actual appkill/제품 downloadergated 연결/변경후전체workspace검증은 미완료다.
