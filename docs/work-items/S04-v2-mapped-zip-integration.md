# S04: Windows FFmpeg용 version2 mapped ZIP

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R11
- 담당/수정 범위: root — archive_layout.rs, archive_manifest.rs, archive_install.rs 및 통합fixture; metrics_recovery — bounded_zip.rs만
- 선행 작업: 공식 WindowsZIP SHA/component inventory; v2 TAR layout 계약

## 사용자 결과

공식 Windows FFmpeg/ffprobe ZIP의 선택 component를 평면 관리 디렉터리에 설치한다. v1 flatZIP 한도와 호환성은 유지한다.

## 공유 계약

ArchiveLayout::Zip { root, mappings }를 version2 명시적으로 추가한다. mapping은 기존 ArchiveMemberMapping exactsource→component며 component hashes가 원권한이다. v2 선택합384MiB, 원ZIP256MiB, 전체선언768MiB, 최대4096entries. 안전한 root하위directory/unselectedregular를 읽어검증하되 선택File capability외filesystem추출은없다. symlink/특수file/경로alias/traversal/duplicate/encrypted/미지원compression/localheader불일치/CRC/SHA실패는거절한다. deadline/cancel/checkpoint/원sourcepin/staging/transaction/원외부source보존은기존정책유지한다.

## 근거와 비용

공식win64ZIP171535354bytes, 실행파일합268793344bytes, license포함268800995bytes로 기존256MiB flat전체한도를357888bytes 이상초과한다. v1한도상향대신 v2명시선택설치예산을재사용한다. 대형파일은stream처리하고추가dependency는없다. 실제cost/installer예약공간및Windowsversion확인은후속실험에기록한다.

## 인수 조건

기존v1허용/reject경계불변, v2nested실제선택/SHA/CRC/미선택경로보존, traversal/link/alias/duplicate/overlap/oversize/cancel실패회귀. 공식ZIP실제inactiveinstall→두version확인→명시select/active삭제거절, 원archive/sentinel보존. 공급license/source/notice채택과전체P0완료는별도다.

## 검증과 handoff

현재rootmarkerWindows전체collector종료전Rust수정은보류하고담당초안을/tmp에서준비한다. 실제검증전완료로표시하지않는다.

### 통합 첫 scoped 결과

root실행layout4시험과boundedZIP10시험PASS. source/archive/component전체메모리적재없이기존streamSHA/선택CRC를재사용한다. 미선택entry는header/local/descriptor/range/type/declaredtotal검증이며payloadCRCdecode는하지않는다. actualofficialWindowsZIP설치collectorwindows-ffmpeg-mapped-zip-install-first실행중. rootaudit에서fileancestor ASCIIcasefold보강을발견해collector종료뒤담당자수정예정이며전체완료로표시하지않는다.
