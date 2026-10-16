# S04: BtbN LGPL FFmpeg 실제 공급 후보 실증

- 상태: done (Linux 공급 후보 실증; 관리 배포 채택 미완료)
- 분류: test
- 우선순위: P0
- 요구사항: R05, R07, R11
- 담당/수정 범위: library_identity — 신규 scripts/ffmpeg_supply_probe.py, 전용 validation/evidence
- 선행 작업: 기존 공급 문서의 고정 archive 후보와 공개 checksum

## 사용자 결과

고정 BtbN LGPL FFmpeg/ffprobe 배포 세트가 합성 처리에 실제 적합한지, archive와 executable digest·저장/실행 비용·license/source 정보로 판단한다. 관리 설치나 공급처 최종 채택은 별도 계약이다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R05·INTEGRATIONS와 기존 공급 증거를 기준으로 공식 README/release API/checksum/legal 자료를 재확인한다. 제품/schema/Cargo/기존 공급문서는 수정하지 않는다. 신규 probe는 bounded download, stream tar allowlist regularfile 추출, 크기/경로/중복/symlink 거절, argv/owned process timeout을 사용한다. 프로젝트/임시 소유 root와 합성 media만 사용한다. 전체 archive checksum은 추출된 executable checksum과 구분한다.

## 인수 조건

- 고정 asset API/checksum/실제 archive size·SHA가 일치한다.
- ffmpeg/ffprobe 및 license/source/build 안내만 경로 탈출 없이 안전 추출한다.
- 정확 executable digest, version/buildconf/license와 합성 encode/probe/decode/cut을 실제 실행한다.
- 다운로드/추출 저장 비용과 합성 실행 CPU/RSS/시간을 측정한다.
- source/notice/attestation 미완비, OS/codec 한계를 후보 채택 판정에 명시한다.

## 검증과 handoff

실제 archive/API/checksum과 component SHA가 일치하고 ffmpeg/ffprobe version/buildconf/license, 합성 MPEG4/AAC encode/probe/decode/기본 정확 자르기가 통과했다. 실행 파일 세트284,847,347bytes와 처리 CPU/RSS를 측정했다. 안전 추출 self-test7개 통과, 문서 검사152개 통과. [검증 결과](../validation/ffmpeg-supply-validation.md), [원시 증거](../validation/ffmpeg-supply-probe.json)에 보존했다. LGPL3+ 후보이며 대응source/의존성notice/trust/3OS/multicomponent 설치가 남아 관리 채택으로 승격하지 않는다. 외부 설치/사용자 media는 사용·삭제하지 않았다. 기존 .git은 보존한다.
