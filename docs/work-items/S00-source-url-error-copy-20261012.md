# S00: 빈 주소와 잘못된 주소 안내 구별

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R04, R08
- 담당/수정 범위: root — crates/app/src/accessible.rs, 이 문서
- 선행 작업: source-prepare native 독립 대조 종료와 source 고정 해제

## 사용자 결과

주소를 입력했으나 형식이 잘못된 경우, 빈 주소 안내 대신 올바른 HTTP/HTTPS 주소를 입력하라는 설명을 본다.

## 계약과 범위

DESIGN.md의 상태와 다음 행동 설명 계약을 따른다. 기존 중앙 catalog의 MEDIA_URL_REQUIRED와 MEDIA_URL_INVALID를 사용한다. SourceUrl 파서와 다운로드 실행 정책·상태 복구 우선순위는 유지한다. 새 schema와 의존성은 없다.

## 인수 조건

실제 Candidate에서 빈 주소는 기존 안내, 잘못된 주소는 형식 오류 안내, 유효한 주소는 후속 준비 조건을 표시한다. 버튼 설명과 본문이 일치하며 입력 초안·focus·caret을 보존한다. 실제 Light/Dark 캡처로 읽을 수 있는지 확인한다.

## 검증과 handoff

root가 source-prepare Light/Dark first의 prepare-invalid PNG를 직접 보았다. 실제 file URL 값이 있는데 본문은 영상 URL을 입력하세요로 표시되었다. native 버튼 설명은 올바른 HTTP 또는 HTTPS 영상 주소를 입력해 주세요여서 서로 다르다. 제품 수정·회귀 검증은 아직 수행하지 않았다.

- source-prepare 독립 Light93578/Dark19383 exit0을 확인하고 원 evidence에 JSON을 보존했다. source 고정을 해제한 뒤 기존 다운로드 gate 회귀가 잘못된 주소에서 올바른 오류 문구를 요구하도록 수정하고 RED를 실행한다. 새 테스트를 구현 복제 형태로 추가하지 않는다.

- RED61708 exit101로 기존 실제 gate 회귀에서 잘못된 URL의 본문 안내가 빈 주소 안내임을 재현했다. 원 로그·source 불변·Job 자연 종료를 보존했다. 빈 trim 결과는 MEDIA_URL_REQUIRED, 비어 있지 않은 parse 실패는 MEDIA_URL_INVALID로 구분했다. 파서·실행 권한·다른 gate 순서는 유지한다. GREEN49248을 실행 중이다.

- GREEN49248 exit0: accessible/CPU 두 Windows harness에서 기존 gate 회귀 각각 통과. 빈 주소·잘못된 주소·저장 위치·도구·미리보기 bypass 조건을 검사했다. source 불변과 소유 Job 자연 종료를 확인했다. 기존 source-first 회귀도 빈/잘못된 주소의 각 정확한 이유를 요구하도록 갱신했다. 전체64499 실행 중이며 최신 native 본문 픽셀 검토는 아직 남는다.

- release_preparation에 /tmp 본문 오류 native 후보를 할당했다. blank/invalid Text 본문·원 native 설명·clip을 실제 관측하며 root가 실행과 직접 PNG 확인을 맡는다. 후보 작성은 구현 인수로 세지 않는다.

- 전체64499와 독립 대조29807 exit0: fmt·clippy·test·release와43 harness/1,165 성공/0 실패/30 ignored 통과. CPU9,714,688B SHA fa71550e5d3c2b5d1f16c9cfd80a7cbcfc9c45af8e6b4d01eef3aaac2c7fa6c8. 직전과 같은 크기이고 새 의존성은 없다. native Light93087/Dark5299를 시작했으며 actual Text/description/fullclip은 결과 취득 후 확인한다.

- 실제 native Light93087/Dark5299 및 독립78220/94887 exit0. 빈/무효 주소의 실제 Text 본문·Button/시작조건 native 설명과 원 identity/clip/Job 자연 종료를 확인했고 invalid에서 옛 빈 주소 본문이 없는 것을 검사했다. root는 Light/Dark first/restart invalid PNG4개를 직접 보아 정확한 안내 전문이 footer 위에 온전히 보임을 확인했다. 각 root-scoped-pixel-review.json과 root-independent-native-review.json에 보존했다. 이 좁은 오류 안내 작업을 완료하며 전체 UI/UX·물리 IME를 완료로 주장하지 않는다.
