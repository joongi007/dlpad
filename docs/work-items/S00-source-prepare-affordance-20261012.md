# S00: 영상 확인 행동의 입력 조건

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R04, R08
- 담당/수정 범위: diagnostic_cache — /tmp 후보; root — crates/app/src/accessible.rs 통합, 이 문서
- 선행 작업: input-focus-error 전체·native source 고정 검사 종료

## 사용자 결과

빈 주소에서 영상 확인을 누르도록 유도하지 않고, 유효한 주소의 명시적 확인 및 실패 후 재시도를 제공한다.

## 계약과 범위

SourceUrl 문법 확인과 실제 영상 준비 성공을 구별한다. 자동 준비·취소·download gate·도구 실행 정책을 바꾸지 않는다. 원 URL ID·caret·Enter 정책을 보존하며 명시적 확인의 enabled 이유를 연결한다. 프로젝트 파일은 root만 수정한다.

## 인수 조건

실제 Candidate의 빈/잘못된 URL에서는 확인 disabled 및 설명을 제공하고 유효 URL에서는 기존 행동에 접근한다. busy·오류 후 재시도·입력 보존을 검증한다. native 클릭 및 캡처는 별도다.

## 검증과 handoff

직접 본 source-first Light/Dark800×640/앱200% 캡처와 현재 code에서 빈 입력의 영상 확인 버튼 enabled를 확인했다. 후보 준비 중이며 제품 수정·회귀 완료는 아니다.

- root는 확인 버튼의 기존 enabled 동작을 유지한 채 입력 조건 helper와 실제 Candidate 회귀를 먼저 추가했다. 빈/잘못된/유효 URL·실패 후 재시도·pending gate와 소유 worker 종료 ACK를 검사한다. Windows RED 회귀62036 실행 중이며 성공을 주장하지 않는다.

- RED62036은 새 회귀를 bounded_font_tests에 잘못 넣어 fixture가 범위 밖이어서 compile 실패했다. root가 기존 Candidate tests 모듈로 옮겼다. 원 실패를 보존하고 실제 버튼 동작을 확인하는 RED-v2 21099를 실행했다. compile 실패를 제품 버그 재현으로 세지 않는다.

- 실제 RED-v2 21099 exit101: 빈 주소의 실제 Button.is_disabled=false인데 true를 요구하여 실패했다. source 불변·소유 Job 자연 종료를 확인했다. root가 명시 확인 버튼과 Enter에 동일 조건을 적용하고 비활성 사유를 tooltip/접근성 description에 연결했다. 기존 자동 준비·취소 정책과 입력 ID는 유지한다. GREEN 검증을 실행한다.

- GREEN31802 exit0: 실제 Windows accessible/CPU 두 harness의 회귀가 각각 통과했다. 빈/잘못된 URL의 disabled+정확한 설명, 유효 URL의 enabled+설명 제거, 원 초안 보존과 실패 후 재시도, pending gate·소유 worker stop ACK를 확인했다. source 불변·Job 자연 종료. rendered busy와 native 픽셀 인수는 아직 남으며 전체 MSVC 검사를 시작했다.

- release_preparation에 /tmp native 후보를 할당했다. 범위는 입력 조건의 실제 Windows Button enabled/설명·초안/입력 identity 보존, private 화면과 원 소유 fixture를 사용하는 관측이다. 제품 파일/공유 Cargo는 root 전용이며 native 실행도 root가 한다. 기존 source-first·진단 시험 파일과 동시 수정하지 않는다.

- 최신 전체57927 및 독립 대조31904 exit0: fmt·clippy·test·release 통과, 43 harness/1,165 성공/0 실패/30 ignored. 실제 로그·current source·GNU 원본과 실행 파일 해시를 대조했다. CPU 실행 파일9,714,688B, SHA874fa33f8bd463b7f6efe2adbc4de8416e7c3abc6854e654c80e2a74d32272c8이며 직전보다1,536B 증가했다. 새 runtime 의존성은 없다. 증거: docs/validation/evidence/windows-msvc-source-prepare-affordance-whole-20261012/root-review.json. UI 전체 인수와 최신 설치 ZIP 생성은 아직 남는다.

- 실제 native Light46386/Dark71903 시작. 빈/무효/유효 URL의 Button 상태·native 설명·원 입력 runtime ID와 재시작을 private desktop에서 확인한다. 기존 소유 fixture의 정확한 SHA를 yt-dlp/FFmpeg에 고정하며 공개 network·실제 영상·새 서버를 사용하지 않는다. fixture 최초 compile provenance 미확인은 유지한다. root 직접 PNG·독립 검토는 결과를 얻은 후 수행한다.

- source-prepare actual native Light46386/Dark71903 exit0. root가 first의 빈/무효/유효 URL 캡처 여섯 장을 직접 확인했다. 영상 확인 버튼은 빈/무효에서 흐린 비활성, 유효에서 정상 활성이다. 잘못된 URL의 본문 안내는 빈 주소 문구여서 별도 S00-source-url-error-copy 작업으로 기록했다. 독립 검토·restart 직접 픽셀 검토는 남는다.

- root가 restart의 빈/무효/유효 URL 캡처 여섯 장도 직접 보았다. 양테마 first/restart12개 PNG의 버튼·입력 가시성과 상태별 활성 외관을 각 root-scoped-pixel-review.json에 기록했다. 잘못된 URL 본문 안내는 실패로 명시하며 전체 UX 성공으로 확장하지 않는다.

- 독립 source-prepare Light93578/Dark19383 exit0: 원 전체/current source·CPU·fixture·생명주기·재시작·URL identity·native Button enabled/설명을 대조했다. exact valid 설명은 빈 값이었다. 자동 미리보기 worker 완료의 직접 receipt는 없으며 앱의 자연 종료와 구별한다. 증거를 각 root-independent-native-review.json에 보존했다. 본문 오류 문구와 전체 UI 시각 인수는 별도 작업이다.
