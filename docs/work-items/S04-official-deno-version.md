# S04: 공식 Deno 버전 출력 실증

- 상태: done
- 분류: experiment
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: root — 이 문서, docs/validation/evidence/deno-official-version-first/
- 선행 작업: 공식 archive/component exact SHA·크기 증거

## 사용자 결과

실제 공식 Deno의 버전 출력 형식과 CLI 비용을 확인해 제품의 typed version 검증 근거를 제공한다. 이것은 제품 다운로드의 runtime 연결 완료가 아니다.

## 계약과 범위

원 ZIP read-only/no-follow FD와 공식 고정 hash를 확인하고 /tmp 새 소유 폴더에 단일 deno member를64KiB stream으로 추출해 정확component SHA/bytes를 재검증한다. 원source는 변경하지 않는다. verified 임시 파일만0700 executable로 argv --version을 실행하며 shell/인증/영상/GUI/활성 전환 없이10초 process deadline·소유processgroup cleanup을 사용한다. stdout/stderr≤8192bytes를 검사한다. 공급 publisher trust/notice 채택은 별도다.

## 인수 조건

실제 firstline version/target 및 exit/경과시간, original source 전후 SHA, 임시파일 자동정리와 미검증 범위를 원 JSON에 기록한다.

## 검증과 handoff

실행을 시작한 뒤 이 별도 scope를 기록했다. 제품과 실험 범위는 명확히 구분한다. 현재 원 결과 수집 중이다.

## 실제 결과

[원 JSON](../validation/evidence/deno-official-version-first/result.json)은 Linux 공식Deno2.9.7 --version exit0/0.817초, actual3줄(deno stable release target/v8/typescript), source 전후SHA동일 및 component95830104 bytes/SHA일치 PASS다. 임시 executable 실행 후 정상회수했고 source를보존했다. 제품process guardian/version API 및 UI·Windows/macOS runtime·독립publisher trust/notice는미검증이다.
